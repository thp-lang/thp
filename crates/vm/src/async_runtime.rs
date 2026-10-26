use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use pcre2::bytes::Regex;
use thp_bytecode::{Function, Instruction, InstructionKind, Terminator};
use thp_diagnostics::Span;
use thp_hir::{
    Builtin, CalledClass, Callee, ClassId, DirectoryBuiltin, FileObjectBuiltin, FunctionId,
    PropertyId, ReflectionBuiltin, Type,
};
use thp_mir::{BlockId, Register};
use thp_runtime::{
    DirectoryPending, DirectoryReadError, DirectoryStart, FileInfoPending, FileInfoStart,
    GeneratorFrame, GeneratorState, GeneratorStatus, OpenFileFlags, OpenFilePending, OpenFileStart,
    RecursiveDirectoryPending, RecursiveDirectoryStart, StackFrame, StreamFileOperation,
    StreamFilePending, StreamFileResult, StreamFileStart, Value, file_info_uses_host,
    start_directory_read, start_file_info, start_open_file, start_recursive_directory_read,
};

use super::{
    ExecutionState, Frame, GENERATOR_EXIT_CODE, RecursiveChild, RuntimeError, RuntimeErrorKind,
    VmError, WalkFrame, catch_exception, directory_entry_name, get_register, is_instance_of_name,
    local_file_uri_path, pcre, runtime, substitute_type_arguments, value_matches,
};

const UNSUBSCRIBE: FunctionId = FunctionId(u32::MAX);
const NOTIFY: FunctionId = FunctionId(u32::MAX - 1);

fn id_integer(id: usize) -> Value {
    Value::integer(i64::try_from(id).expect("request-local IDs fit signed integers"))
}

#[derive(Default)]
pub(super) struct AsyncRuntime {
    registration: Option<Registration>,
    sources: Vec<Source>,
    ready: VecDeque<usize>,
    current: Option<usize>,
    next_subscription: i64,
    in_callback: bool,
    notifications: VecDeque<usize>,
    root_wait: Option<(usize, Option<usize>)>,
    root_winner: Option<bool>,
    timers: BinaryHeap<Reverse<(Instant, usize)>>,
}

enum Registration {
    Default,
    Custom(Value),
}

#[allow(clippy::struct_excessive_bools)]
struct Source {
    handle: Value,
    kind: SourceKind,
    result: Option<Result<Value, Value>>,
    failure_trace: Vec<StackFrame>,
    subscribers: Vec<Subscription>,
    started: bool,
    suspended: bool,
    cancel_called: bool,
    cancel_requested: bool,
    cancelled: bool,
    observed: bool,
    frames: Vec<TaskFrame>,
    waiting: Option<Wait>,
}

struct TaskFrame {
    function: FunctionId,
    frame: Frame,
    destination: Option<Register>,
    caller_span: Span,
    validate_countable: bool,
    return_override: Option<Value>,
    reflection_return: Option<Type>,
    generator: Option<GeneratorActivation>,
    collection: Option<Box<CollectionContinuation>>,
    iterator: Option<Box<IteratorContinuation>>,
    native_iterators: Vec<NativeIteratorContinuation>,
}

struct GeneratorActivation {
    iterator: Value,
    state: GeneratorState,
    close_marker: Option<Value>,
}

enum CollectionInput {
    Vector(Value),
    Map(Value),
}

struct CollectionContinuation {
    builtin: Builtin,
    callback: Value,
    input: CollectionInput,
    index: usize,
    output: Value,
    instruction: Instruction,
    pending: bool,
}

impl CollectionContinuation {
    fn len(&self) -> usize {
        match &self.input {
            CollectionInput::Vector(values) => values.vector_values().expect("vector source").len(),
            CollectionInput::Map(entries) => entries.map_entries().expect("map source").len(),
        }
    }

    fn callback_arguments(&self) -> Vec<Value> {
        match &self.input {
            CollectionInput::Vector(values) => {
                vec![values.vector_values().expect("vector source")[self.index].clone()]
            }
            CollectionInput::Map(entries) => {
                let (key, value) = &entries.map_entries().expect("map source")[self.index];
                vec![value.clone(), key.clone()]
            }
        }
    }

    fn receive(&mut self, mapped: Value) -> Result<(), RuntimeErrorKind> {
        match (&self.input, self.builtin) {
            (CollectionInput::Vector(_), Builtin::VectorMap) => {
                self.output.vector_push(mapped)?;
            }
            (CollectionInput::Vector(values), Builtin::VectorFilter) => {
                if mapped.as_bool() == Some(true) {
                    self.output.vector_push(
                        values.vector_values().expect("vector source")[self.index].clone(),
                    )?;
                }
            }
            (CollectionInput::Map(entries), Builtin::MapTransform) => {
                self.output.set_index(
                    &entries.map_entries().expect("map source")[self.index].0,
                    mapped,
                )?;
            }
            (CollectionInput::Map(entries), Builtin::MapFilter) => {
                if mapped.as_bool() == Some(true) {
                    let (key, value) = &entries.map_entries().expect("map source")[self.index];
                    self.output.set_index(key, value.clone())?;
                }
            }
            _ => unreachable!("collection continuation has a callback operation"),
        }
        self.index += 1;
        self.pending = false;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum IteratorPhase {
    Valid,
    Key,
    Value,
    Callback,
    Advance,
}

struct IteratorContinuation {
    builtin: Builtin,
    iterator: Value,
    callback: Option<Value>,
    output: Value,
    count: i64,
    key: Option<Value>,
    value: Option<Value>,
    stop: bool,
    phase: IteratorPhase,
    scratch: Register,
    instruction: Instruction,
    pending: bool,
    restore_instruction: bool,
}

impl IteratorContinuation {
    fn receive(&mut self, value: Value) -> Result<bool, RuntimeErrorKind> {
        match self.phase {
            IteratorPhase::Valid => {
                if value.as_bool() != Some(true) {
                    return Ok(true);
                }
                self.phase = match self.builtin {
                    Builtin::IteratorCount => IteratorPhase::Advance,
                    Builtin::IteratorToMap => IteratorPhase::Key,
                    _ => IteratorPhase::Value,
                };
                if self.builtin == Builtin::IteratorCount {
                    self.increment()?;
                }
            }
            IteratorPhase::Key => {
                self.key = Some(value);
                self.phase = if self.builtin == Builtin::IteratorApply {
                    IteratorPhase::Callback
                } else {
                    IteratorPhase::Value
                };
            }
            IteratorPhase::Value => match self.builtin {
                Builtin::IteratorToVector => {
                    self.output.vector_push(value)?;
                    self.increment()?;
                    self.phase = IteratorPhase::Advance;
                }
                Builtin::IteratorToMap => {
                    self.output
                        .set_index(&self.key.take().expect("iterator key"), value)?;
                    self.increment()?;
                    self.phase = IteratorPhase::Advance;
                }
                Builtin::IteratorApply => {
                    self.value = Some(value);
                    self.phase = IteratorPhase::Key;
                }
                _ => unreachable!(),
            },
            IteratorPhase::Callback => {
                self.increment()?;
                self.stop = value.as_bool() == Some(false);
                self.phase = IteratorPhase::Advance;
            }
            IteratorPhase::Advance => {
                if self.stop {
                    return Ok(true);
                }
                self.phase = IteratorPhase::Valid;
            }
        }
        Ok(false)
    }

    fn increment(&mut self) -> Result<(), RuntimeErrorKind> {
        self.count = self.count.checked_add(1).ok_or_else(|| {
            RuntimeErrorKind::Arithmetic("iterator count exceeds int range".to_owned())
        })?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum FilterPhase {
    Rewind,
    CheckForAdvance,
    CheckValid,
    ReadValue,
    ReadKey,
    Accept,
    AcceptParentChild,
    SkipAdvance,
    FinalValue,
    ReadChild,
    ReadWrappedValue,
    Advance,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterKind {
    Plain,
    Recursive,
}

struct FilterContinuation {
    builtin: Builtin,
    wrapper: Value,
    inner: Value,
    kind: FilterKind,
    phase: FilterPhase,
    value: Option<Value>,
    key: Option<Value>,
    child: Option<Value>,
    scratch: Register,
    instruction: Instruction,
    pending: bool,
    restore_instruction: bool,
    nested: bool,
}

enum NativeIteratorContinuation {
    Filter(FilterContinuation),
    Limit(LimitContinuation),
    Append(AppendContinuation),
    Caching(CachingContinuation),
    Infinite(InfiniteContinuation),
    Regex(RegexContinuation),
    Walk(WalkContinuation),
    LimitSeek(LimitSeekContinuation),
}

impl NativeIteratorContinuation {
    fn scratch(&self) -> Register {
        match self {
            Self::Filter(state) => state.scratch,
            Self::Limit(state) => state.scratch,
            Self::Append(state) => state.scratch,
            Self::Caching(state) => state.scratch,
            Self::Infinite(state) => state.scratch,
            Self::Regex(state) => state.scratch,
            Self::Walk(state) => state.scratch,
            Self::LimitSeek(state) => state.scratch,
        }
    }
}

#[derive(Clone, Copy)]
enum LimitPhase {
    Rewind,
    SeekValid,
    SeekAdvance,
    Valid,
    Guard,
    FinalValue,
    Advance,
}

struct LimitContinuation {
    builtin: Builtin,
    wrapper: Value,
    inner: Value,
    phase: LimitPhase,
    position: i64,
    remaining: i64,
    scratch: Register,
    instruction: Instruction,
    pending: bool,
    restore_instruction: bool,
    nested: bool,
}

#[derive(Clone, Copy)]
enum AppendPhase {
    Rewind,
    CheckValid { after_advance: bool },
    RewindNext { after_advance: bool },
    FinalValue,
    Advance,
}

struct AppendContinuation {
    builtin: Builtin,
    wrapper: Value,
    iterators: Value,
    index: i64,
    phase: AppendPhase,
    scratch: Register,
    instruction: Instruction,
    pending: bool,
    restore_instruction: bool,
    nested: bool,
}

impl AppendContinuation {
    fn inner(&self) -> Option<Value> {
        usize::try_from(self.index).ok().and_then(|index| {
            self.iterators
                .vector_values()
                .expect("append iterator source")
                .get(index)
                .cloned()
        })
    }
}

#[derive(Clone, Copy)]
enum CachingPhase {
    Rewind,
    PullValid,
    PullKey,
    PullValue,
    PullAdvance,
    Store,
    StoreStringCall,
}

struct CachingContinuation {
    wrapper: Value,
    inner: Value,
    phase: CachingPhase,
    pull: u32,
    key: Option<Value>,
    value: Option<Value>,
    scratch: Register,
    instruction: Instruction,
    pending: bool,
    restore_instruction: bool,
    nested: bool,
}

#[derive(Clone, Copy)]
enum InfinitePhase {
    FetchSource,
    ResolveFetched,
    FetchAggregate,
    RewindFetched,
    ValidFetched,
    CheckInnerValid { after_advance: bool },
    AdvanceInner,
    FinalMethod,
}

struct InfiniteContinuation {
    builtin: Builtin,
    wrapper: Value,
    inner: Option<Value>,
    fetched: Option<Value>,
    aggregate_depth: u32,
    phase: InfinitePhase,
    scratch: Register,
    instruction: Instruction,
    pending: bool,
    restore_instruction: bool,
    nested: bool,
}

#[derive(Clone, Copy)]
enum RegexPhase {
    Rewind,
    CheckForAdvance,
    CheckValid,
    ReadKey,
    ReadEntry,
    ReadNodeValue,
    Evaluate,
    ReadChildren,
    Decide,
    SkipAdvance,
    AdvanceInner,
    FinalKey,
}

struct RegexScanOptions {
    mode: i64,
    invert: bool,
    flags: i64,
    replacement: Value,
    recursive: bool,
}

struct RegexContinuation {
    builtin: Builtin,
    wrapper: Value,
    inner: Value,
    regex: Option<Regex>,
    options: Option<RegexScanOptions>,
    phase: RegexPhase,
    key: Option<Value>,
    entry: Option<Value>,
    value: Option<Value>,
    child: Option<Value>,
    matched: Option<bool>,
    scratch: Register,
    instruction: Instruction,
    pending: bool,
    restore_instruction: bool,
    nested: bool,
}

#[derive(Clone, Copy)]
enum WalkPhase {
    BeginIteration,
    RewindRoot,
    Seek,
    CheckValid,
    ReadEntry,
    ReadChild,
    BeginChildren,
    RewindChild,
    EndChildren,
    PublishKey,
    PublishValue,
    NextElement,
    EndIteration,
    AdvanceCursor,
}

struct WalkContinuation {
    wrapper: Value,
    frames: Vec<WalkFrame>,
    mode: i64,
    max_depth: i64,
    phase: WalkPhase,
    key: Option<Value>,
    scratch: Register,
    instruction: Instruction,
    pending: bool,
    restore_instruction: bool,
    nested: bool,
}

#[derive(Clone, Copy)]
enum LimitSeekPhase {
    Rewind,
    CheckValid,
    Advance,
    FinalValid,
}

struct LimitSeekContinuation {
    wrapper: Value,
    position: i64,
    index: i64,
    phase: LimitSeekPhase,
    scratch: Register,
    instruction: Instruction,
    pending: bool,
    restore_instruction: bool,
    nested: bool,
}

enum Wait {
    Yield {
        span: Span,
        destination: Option<Register>,
    },
    Delay {
        deadline: Instant,
        span: Span,
        destination: Option<Register>,
    },
    Await {
        target: usize,
        cancellation: Option<usize>,
        winner: Option<bool>,
        span: Span,
        destination: Option<Register>,
    },
    File {
        pending: StreamFilePending,
        completion: FileCompletion,
        span: Span,
        destination: Option<Register>,
    },
    Open {
        pending: OpenFilePending,
        completion: OpenCompletion,
        span: Span,
        destination: Option<Register>,
    },
    Info {
        pending: FileInfoPending,
        span: Span,
        destination: Option<Register>,
    },
    Directory {
        pending: DirectoryPending,
        completion: Box<DirectoryCompletion>,
        span: Span,
        destination: Option<Register>,
    },
    RecursiveDirectory {
        pending: RecursiveDirectoryPending,
        completion: Box<RecursiveDirectoryCompletion>,
        span: Span,
        destination: Option<Register>,
    },
    WorkerRetry {
        span: Span,
        retry_at: Instant,
    },
    FileRetry {
        handle: Value,
        span: Span,
        retry_at: Option<Instant>,
    },
}

struct OpenCompletion {
    path: Vec<u8>,
    class: ClassId,
    flags: OpenFileFlags,
    object: Option<Value>,
}

struct DirectoryCompletion {
    operation: DirectoryBuiltin,
    arguments: Vec<Value>,
    instruction: Instruction,
    directory: String,
}

struct RecursiveDirectoryCompletion {
    object: Value,
    info: Value,
    path: Vec<u8>,
    instruction: Instruction,
}

enum FileCompletion {
    Stream(Builtin),
    FileObject {
        operation: FileObjectBuiltin,
        object: Value,
        line: Option<i64>,
    },
}

impl Wait {
    fn span(&self) -> Span {
        match self {
            Self::Yield { span, .. }
            | Self::Delay { span, .. }
            | Self::Await { span, .. }
            | Self::File { span, .. }
            | Self::Open { span, .. }
            | Self::Info { span, .. }
            | Self::Directory { span, .. }
            | Self::RecursiveDirectory { span, .. }
            | Self::WorkerRetry { span, .. }
            | Self::FileRetry { span, .. } => *span,
        }
    }

    fn destination(&self) -> Option<Register> {
        match self {
            Self::Yield { destination, .. }
            | Self::Delay { destination, .. }
            | Self::Await { destination, .. }
            | Self::File { destination, .. }
            | Self::Open { destination, .. }
            | Self::Info { destination, .. }
            | Self::Directory { destination, .. }
            | Self::RecursiveDirectory { destination, .. } => *destination,
            Self::FileRetry { .. } | Self::WorkerRetry { .. } => None,
        }
    }
}

enum TaskOutcome {
    Suspended(Wait),
    Completed(Value),
}

enum CallOutcome {
    Continue,
    Suspend(Wait),
}

enum SourceKind {
    Coroutine {
        callback: Option<Value>,
        arguments: Vec<Value>,
    },
    Timeout,
    External {
        unsubscribe: Option<Value>,
    },
}

struct Subscription {
    id: i64,
    callback: Value,
}

impl ExecutionState<'_, '_> {
    pub(super) fn execute_root(&mut self) -> Result<Value, VmError> {
        let entry = self.program.entry;
        let function = &self.program.functions[entry.0 as usize];
        let instruction = Instruction {
            kind: InstructionKind::Constant(thp_mir::Constant::Null),
            destination: None,
            ty: None,
            span: function.span,
        };
        let mut frames = vec![TaskFrame {
            function: entry,
            frame: self.new_frame(entry, Vec::new(), None),
            destination: None,
            caller_span: function.span,
            validate_countable: false,
            return_override: None,
            reflection_return: None,
            generator: None,
            collection: None,
            iterator: None,
            native_iterators: Vec::new(),
        }];
        loop {
            let mut wait = match self.step_task(&mut frames)? {
                TaskOutcome::Completed(value) => return Ok(value),
                TaskOutcome::Suspended(wait) => wait,
            };
            if let Wait::FileRetry {
                handle,
                span,
                retry_at,
            } = &wait
            {
                while handle
                    .stream_file_busy()
                    .map_err(|kind| runtime(kind, *span))?
                    || retry_at.is_some_and(|deadline| Instant::now() < deadline)
                {
                    if !self.run_ready(0, function, &instruction)? {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    self.tick(*span)?;
                }
                continue;
            }
            if let Wait::WorkerRetry { retry_at, span } = &wait {
                while Instant::now() < *retry_at {
                    if !self.run_ready(0, function, &instruction)? {
                        std::thread::sleep(retry_at.saturating_duration_since(Instant::now()));
                    }
                    self.tick(*span)?;
                }
                continue;
            }
            let wait_span = wait.span();
            let resume = match &mut wait {
                Wait::Yield { .. } => {
                    self.run_ready(0, function, &instruction)?;
                    Ok(Value::NULL)
                }
                Wait::Delay { deadline, .. } => {
                    while Instant::now() < *deadline {
                        if !self.run_ready(0, function, &instruction)? {
                            std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                        }
                        self.tick(wait_span)?;
                    }
                    Ok(Value::NULL)
                }
                Wait::Await {
                    target,
                    cancellation,
                    ..
                } => {
                    self.async_runtime.root_wait = Some((*target, *cancellation));
                    self.async_runtime.root_winner =
                        if self.async_runtime.sources[*target - 1].result.is_some() {
                            Some(true)
                        } else if cancellation
                            .is_some_and(|id| self.async_runtime.sources[id - 1].result.is_some())
                        {
                            Some(false)
                        } else {
                            None
                        };
                    let mut deadlock = None;
                    while self.async_runtime.root_winner.is_none() {
                        if !self.run_ready(0, function, &instruction)?
                            && self.async_runtime.root_winner.is_none()
                        {
                            if let Some(deadline) = self.next_wakeup_deadline() {
                                std::thread::sleep(
                                    deadline.saturating_duration_since(Instant::now()),
                                );
                            } else {
                                deadlock = Some(self.native_exception(
                                    "AsyncDeadlockException",
                                    b"no runnable coroutine or wakeup".to_vec(),
                                    None,
                                    0,
                                    wait_span,
                                ));
                                break;
                            }
                        }
                        self.tick(wait_span)?;
                    }
                    let winner = self.async_runtime.root_winner.take();
                    self.async_runtime.root_wait = None;
                    self.release_external_if_unused(*target, 0, function, &instruction)?;
                    if let Some(cancellation) = cancellation {
                        self.release_external_if_unused(*cancellation, 0, function, &instruction)?;
                    }
                    if let Some(deadlock) = deadlock {
                        Err(deadlock)
                    } else if winner.expect("root has winner") {
                        self.read_result(*target, wait_span)
                    } else {
                        let cause = self.cancellation_cause(*cancellation);
                        let error = self.native_exception(
                            "WaitCancelledException",
                            b"wait cancelled".to_vec(),
                            None,
                            0,
                            wait_span,
                        );
                        if let VmError::Thrown { value, .. } = &error
                            && let Some(cause) = cause
                        {
                            value
                                .append_previous(cause)
                                .map_err(|kind| runtime(kind, wait_span))?;
                        }
                        Err(error)
                    }
                }
                Wait::File {
                    pending,
                    completion,
                    span,
                    ..
                } => {
                    while !pending.poll() {
                        if !self.run_ready(0, function, &instruction)? {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        self.tick(*span)?;
                    }
                    self.file_result_value(
                        completion,
                        pending.take_result().expect("file result is ready"),
                        *span,
                    )
                }
                Wait::Open {
                    pending,
                    completion,
                    span,
                    ..
                } => {
                    while !pending.poll() {
                        if !self.run_ready(0, function, &instruction)? {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        self.tick(*span)?;
                    }
                    self.file_open_result_value(
                        completion,
                        pending.take_result().expect("file open result is ready"),
                        *span,
                    )
                }
                Wait::Info { pending, span, .. } => {
                    while !pending.poll() {
                        if !self.run_ready(0, function, &instruction)? {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        self.tick(*span)?;
                    }
                    self.file_info_value(
                        pending.take_result().expect("file info result is ready"),
                        *span,
                    )
                }
                Wait::Directory {
                    pending,
                    completion,
                    span,
                    ..
                } => {
                    while !pending.poll() {
                        if !self.run_ready(0, function, &instruction)? {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        self.tick(*span)?;
                    }
                    self.directory_result_value(
                        completion,
                        pending.take_result().expect("directory result is ready"),
                        *span,
                    )
                }
                Wait::RecursiveDirectory {
                    pending,
                    completion,
                    span,
                    ..
                } => {
                    while !pending.poll() {
                        if !self.run_ready(0, function, &instruction)? {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        self.tick(*span)?;
                    }
                    self.recursive_directory_result_value(
                        completion,
                        pending
                            .take_result()
                            .expect("recursive directory result is ready"),
                        *span,
                    )
                }
                Wait::FileRetry { .. } | Wait::WorkerRetry { .. } => {
                    unreachable!("handled above")
                }
            };
            match resume {
                Ok(value) => {
                    let frame = &mut frames.last_mut().expect("root has suspended frame").frame;
                    if let Some(destination) = wait.destination() {
                        frame.registers[destination.0 as usize] = Some(value);
                    }
                    frame.next_instruction += 1;
                }
                Err(error) => self.task_error(&mut frames, error)?,
            }
        }
    }

    pub(super) fn async_builtin(
        &mut self,
        builtin: Builtin,
        arguments: &[Value],
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        let span = instruction.span;
        if self.async_runtime.in_callback
            && matches!(
                builtin,
                Builtin::SchedulerRegisterDefault
                    | Builtin::SchedulerRegister
                    | Builtin::Async
                    | Builtin::Await
                    | Builtin::Suspend
                    | Builtin::Delay
                    | Builtin::Timeout
                    | Builtin::CoroutineCancel
                    | Builtin::TimeoutCancel
                    | Builtin::CoroutineSubscribe
                    | Builtin::TimeoutSubscribe
            )
        {
            return Err(self.async_error(
                "Async callback cannot reenter or suspend the scheduler",
                span,
            ));
        }
        match builtin {
            Builtin::SchedulerIsRegistered => {
                Ok(Value::bool(self.async_runtime.registration.is_some()))
            }
            Builtin::SchedulerRegisterDefault | Builtin::SchedulerRegister => {
                if self.async_runtime.registration.is_some() {
                    return Err(self.async_error("scheduler is already registered", span));
                }
                self.async_runtime.registration =
                    Some(if builtin == Builtin::SchedulerRegisterDefault {
                        Registration::Default
                    } else {
                        Registration::Custom(arguments[0].clone())
                    });
                Ok(Value::NULL)
            }
            Builtin::Async => {
                self.require_scheduler(span)?;
                let Type::Nominal {
                    arguments: types, ..
                } = instruction.ty.as_ref().expect("verified async type")
                else {
                    unreachable!("verified async result is Coroutine<T>")
                };
                let class = self.classes_by_name["Coroutine"];
                let handle = Value::try_typed_object(class, types.clone(), 1)
                    .map_err(|kind| runtime(kind, span))?;
                let id = self.async_runtime.sources.len() + 1;
                handle
                    .set_property(PropertyId(0), id_integer(id))
                    .map_err(|kind| runtime(kind, span))?;
                self.async_runtime.sources.push(Source {
                    handle: handle.clone(),
                    kind: SourceKind::Coroutine {
                        callback: Some(arguments[0].clone()),
                        arguments: arguments[1..].to_vec(),
                    },
                    result: None,
                    failure_trace: Vec::new(),
                    subscribers: Vec::new(),
                    started: false,
                    suspended: false,
                    cancel_called: false,
                    cancel_requested: false,
                    cancelled: false,
                    observed: false,
                    frames: Vec::new(),
                    waiting: None,
                });
                self.async_runtime.ready.push_back(id);
                Ok(handle)
            }
            Builtin::Timeout => {
                self.require_scheduler(span)?;
                let milliseconds = arguments[0].as_int().expect("verified timeout duration");
                if milliseconds <= 0 {
                    return Err(self.async_error("timeout duration must be positive", span));
                }
                let deadline = Instant::now()
                    .checked_add(Duration::from_millis(
                        u64::try_from(milliseconds).expect("positive duration"),
                    ))
                    .ok_or_else(|| self.async_error("timeout duration is too large", span))?;
                let handle = Value::try_typed_object(self.classes_by_name["Timeout"], vec![], 1)
                    .map_err(|kind| runtime(kind, span))?;
                let id = self.async_runtime.sources.len() + 1;
                handle
                    .set_property(PropertyId(0), id_integer(id))
                    .map_err(|kind| runtime(kind, span))?;
                self.async_runtime.sources.push(Source {
                    handle: handle.clone(),
                    kind: SourceKind::Timeout,
                    result: None,
                    failure_trace: Vec::new(),
                    subscribers: Vec::new(),
                    started: true,
                    suspended: true,
                    cancel_called: false,
                    cancel_requested: false,
                    cancelled: false,
                    observed: false,
                    frames: Vec::new(),
                    waiting: None,
                });
                self.async_runtime.timers.push(Reverse((deadline, id)));
                Ok(handle)
            }
            Builtin::Suspend | Builtin::Delay => {
                self.require_scheduler(span)?;
                self.check_async_cancellation(span)?;
                let milliseconds = if builtin == Builtin::Delay {
                    arguments[0].as_int().expect("verified delay duration")
                } else {
                    0
                };
                if milliseconds < 0 {
                    return Err(self.async_error("delay duration cannot be negative", span));
                }
                let deadline = Instant::now()
                    .checked_add(Duration::from_millis(
                        u64::try_from(milliseconds).expect("nonnegative duration"),
                    ))
                    .ok_or_else(|| self.async_error("delay duration is too large", span))?;
                if let Some(id) = self.async_runtime.current {
                    self.async_runtime.sources[id - 1].suspended = true;
                }
                if milliseconds == 0 {
                    if !self.async_runtime.ready.is_empty() {
                        self.run_ready(depth, calling_function, instruction)?;
                    }
                } else {
                    while Instant::now() < deadline {
                        if !self.run_ready(depth, calling_function, instruction)? {
                            let remaining = deadline.saturating_duration_since(Instant::now());
                            std::thread::sleep(remaining);
                        }
                        self.tick(span)?;
                        self.check_async_cancellation(span)?;
                    }
                }
                if let Some(id) = self.async_runtime.current {
                    self.async_runtime.sources[id - 1].suspended = false;
                }
                self.check_async_cancellation(span)?;
                Ok(Value::NULL)
            }
            Builtin::Await => self.await_source(
                &arguments[0],
                &arguments[1],
                depth,
                calling_function,
                instruction,
            ),
            Builtin::CoroutineCancel | Builtin::TimeoutCancel => {
                let id = self.source_id(&arguments[0], span)?;
                self.cancel_source(id, depth, calling_function, instruction)?;
                Ok(Value::NULL)
            }
            Builtin::CoroutineGetId => Ok(id_integer(self.source_id(&arguments[0], span)?)),
            Builtin::CoroutineIsStarted => {
                let id = self.source_id(&arguments[0], span)?;
                Ok(Value::bool(self.async_runtime.sources[id - 1].started))
            }
            Builtin::CoroutineIsQueued => {
                let id = self.source_id(&arguments[0], span)?;
                Ok(Value::bool(self.async_runtime.ready.contains(&id)))
            }
            Builtin::CoroutineIsRunning => {
                let id = self.source_id(&arguments[0], span)?;
                Ok(Value::bool(
                    self.async_runtime.current == Some(id)
                        && !self.async_runtime.sources[id - 1].suspended,
                ))
            }
            Builtin::CoroutineIsSuspended => {
                let id = self.source_id(&arguments[0], span)?;
                Ok(Value::bool(self.async_runtime.sources[id - 1].suspended))
            }
            Builtin::CoroutineIsCompleted | Builtin::TimeoutIsCompleted => {
                let id = self.source_id(&arguments[0], span)?;
                Ok(Value::bool(
                    self.async_runtime.sources[id - 1].result.is_some(),
                ))
            }
            Builtin::CoroutineIsCancelled | Builtin::TimeoutIsCancelled => {
                let id = self.source_id(&arguments[0], span)?;
                Ok(Value::bool(self.async_runtime.sources[id - 1].cancelled))
            }
            Builtin::CoroutineResult | Builtin::TimeoutResult => {
                let id = self.source_id(&arguments[0], span)?;
                self.read_result(id, span)
            }
            Builtin::CoroutineSubscribe | Builtin::TimeoutSubscribe => {
                let id = self.source_id(&arguments[0], span)?;
                self.subscribe_source(
                    id,
                    arguments[1].clone(),
                    depth,
                    calling_function,
                    instruction,
                )
            }
            _ => unreachable!("non-async builtin"),
        }
    }

    fn async_error(&self, message: &str, span: Span) -> VmError {
        self.native_exception("Error", message.as_bytes().to_vec(), None, 0, span)
    }

    fn fatal_async_error(&self, error: VmError) -> VmError {
        let VmError::Thrown { value, span, trace } = error else {
            return error;
        };
        let class = value
            .class_id()
            .and_then(|id| self.program.classes.get(id.0 as usize))
            .map_or_else(|| "<non-exception>".to_owned(), |class| class.name.clone());
        let message = value
            .exception_message()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_default();
        let mut failure =
            RuntimeError::new(RuntimeErrorKind::UncaughtException { class, message }, span);
        failure.trace = trace;
        VmError::Runtime(failure)
    }

    fn require_scheduler(&self, span: Span) -> Result<(), VmError> {
        if self.async_runtime.registration.is_none() {
            Err(self.async_error("scheduler is not registered", span))
        } else {
            Ok(())
        }
    }

    fn source_id(&self, handle: &Value, span: Span) -> Result<usize, VmError> {
        let id = handle
            .property(PropertyId(0))
            .ok()
            .and_then(|id| id.as_int())
            .and_then(|id| usize::try_from(id).ok())
            .ok_or_else(|| self.async_error("invalid Async handle", span))?;
        if id == 0
            || id > self.async_runtime.sources.len()
            || self.async_runtime.sources[id - 1].handle != *handle
        {
            return Err(self.async_error("Async handle belongs to another request", span));
        }
        Ok(id)
    }

    fn completion_id(
        &mut self,
        handle: &Value,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<usize, VmError> {
        if let Some(index) = self
            .async_runtime
            .sources
            .iter()
            .position(|source| source.handle == *handle)
        {
            let id = index + 1;
            if matches!(
                self.async_runtime.sources[index].kind,
                SourceKind::External { unsubscribe: None }
            ) && self.async_runtime.sources[index].result.is_none()
            {
                let completed = self.call_async_method(
                    handle,
                    "isCompleted",
                    vec![],
                    depth,
                    calling_function,
                    instruction,
                )?;
                if completed.as_bool() == Some(true) {
                    self.settle_external(id, depth, calling_function, instruction)?;
                } else {
                    self.require_scheduler(instruction.span)?;
                    self.subscribe_external(id, depth, calling_function, instruction)?;
                }
            }
            return Ok(id);
        }
        let completed = self.call_async_method(
            handle,
            "isCompleted",
            vec![],
            depth,
            calling_function,
            instruction,
        )?;
        let Some(completed) = completed.as_bool() else {
            return Err(self.async_error("isCompleted() must return bool", instruction.span));
        };
        if !completed {
            self.require_scheduler(instruction.span)?;
        }
        let id = self.async_runtime.sources.len() + 1;
        self.async_runtime.sources.push(Source {
            handle: handle.clone(),
            kind: SourceKind::External { unsubscribe: None },
            result: None,
            failure_trace: Vec::new(),
            subscribers: Vec::new(),
            started: true,
            suspended: !completed,
            cancel_called: false,
            cancel_requested: false,
            cancelled: false,
            observed: false,
            frames: Vec::new(),
            waiting: None,
        });
        if completed {
            self.settle_external(id, depth, calling_function, instruction)?;
        } else {
            self.subscribe_external(id, depth, calling_function, instruction)?;
        }
        Ok(id)
    }

    fn subscribe_external(
        &mut self,
        id: usize,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<(), VmError> {
        let handle = self.async_runtime.sources[id - 1].handle.clone();
        let notify = Value::try_closure(NOTIFY, vec![id_integer(id)])
            .map_err(|kind| runtime(kind, instruction.span))?;
        let unsubscribe = self.call_async_method(
            &handle,
            "subscribe",
            vec![notify],
            depth,
            calling_function,
            instruction,
        )?;
        if let SourceKind::External { unsubscribe: slot } =
            &mut self.async_runtime.sources[id - 1].kind
        {
            *slot = Some(unsubscribe);
        }
        Ok(())
    }

    fn release_external_if_unused(
        &mut self,
        id: usize,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<(), VmError> {
        if self
            .async_runtime
            .root_wait
            .is_some_and(|(target, cancellation)| target == id || cancellation == Some(id))
        {
            return Ok(());
        }
        if self.async_runtime.sources.iter().any(|source| {
            matches!(source.waiting, Some(Wait::Await { target, cancellation, .. })
                if target == id || cancellation == Some(id))
        }) {
            return Ok(());
        }
        let unsubscribe = if let SourceKind::External { unsubscribe } =
            &mut self.async_runtime.sources[id - 1].kind
        {
            unsubscribe.take()
        } else {
            None
        };
        if let Some(unsubscribe) = unsubscribe {
            self.call_async_callback(&unsubscribe, depth, calling_function, instruction)?;
        }
        Ok(())
    }

    fn settle_external(
        &mut self,
        id: usize,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<(), VmError> {
        if self.async_runtime.sources[id - 1].result.is_some() {
            return Ok(());
        }
        let handle = self.async_runtime.sources[id - 1].handle.clone();
        let result = match self.call_async_method(
            &handle,
            "result",
            vec![],
            depth,
            calling_function,
            instruction,
        ) {
            Ok(value) => Ok(value),
            Err(VmError::Thrown { value, trace, .. }) => {
                self.async_runtime.sources[id - 1].failure_trace = trace;
                Err(value)
            }
            Err(error) => return Err(error),
        };
        self.finish_source(id, result, depth, calling_function, instruction)
    }

    fn process_notifications(
        &mut self,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<(), VmError> {
        while let Some(id) = self.async_runtime.notifications.pop_front() {
            if matches!(
                self.async_runtime.sources[id - 1].kind,
                SourceKind::External { .. }
            ) {
                self.settle_external(id, depth, calling_function, instruction)?;
            }
        }
        Ok(())
    }

    fn check_async_cancellation(&mut self, span: Span) -> Result<(), VmError> {
        if let Some(id) = self.async_runtime.current {
            let source = &mut self.async_runtime.sources[id - 1];
            if source.cancel_requested {
                source.cancel_requested = false;
                return Err(self.native_exception(
                    "CancellationException",
                    b"coroutine cancelled".to_vec(),
                    None,
                    0,
                    span,
                ));
            }
        }
        Ok(())
    }

    fn read_result(&mut self, id: usize, span: Span) -> Result<Value, VmError> {
        let source = &mut self.async_runtime.sources[id - 1];
        if source.result.is_some() && matches!(source.kind, SourceKind::Coroutine { .. }) {
            source.observed = true;
        }
        match &source.result {
            Some(Ok(value)) => Ok(value.clone()),
            Some(Err(value)) => Err(VmError::Thrown {
                value: value.clone(),
                span,
                trace: source.failure_trace.clone(),
            }),
            None => Err(self.async_error("completion is pending", span)),
        }
    }

    fn cancellation_cause(&mut self, id: Option<usize>) -> Option<Value> {
        let source = &mut self.async_runtime.sources[id? - 1];
        if matches!(source.kind, SourceKind::Coroutine { .. }) {
            source.observed = true;
        }
        source.result.as_ref()?.as_ref().err().cloned()
    }

    fn subscribe_source(
        &mut self,
        id: usize,
        callback: Value,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        let subscription = self.async_runtime.next_subscription;
        self.async_runtime.next_subscription = subscription
            .checked_add(1)
            .ok_or_else(|| self.async_error("too many subscriptions", instruction.span))?;
        if self.async_runtime.sources[id - 1].result.is_some() {
            self.call_async_callback(&callback, depth, calling_function, instruction)?;
        } else {
            self.async_runtime.sources[id - 1]
                .subscribers
                .push(Subscription {
                    id: subscription,
                    callback,
                });
        }
        Value::try_closure(
            UNSUBSCRIBE,
            vec![id_integer(id), Value::integer(subscription)],
        )
        .map_err(|kind| runtime(kind, instruction.span))
    }

    fn call_async_callback(
        &mut self,
        callback: &Value,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<(), VmError> {
        let previous = self.async_runtime.in_callback;
        self.async_runtime.in_callback = true;
        let result = self.invoke_callback(callback, vec![], depth, calling_function, instruction);
        self.async_runtime.in_callback = previous;
        result
            .map(|_| ())
            .map_err(|error| self.fatal_async_error(error))
    }

    pub(super) fn invoke_async_callback(
        &mut self,
        function: FunctionId,
        captures: &[Value],
    ) -> Option<Result<Value, VmError>> {
        if function == NOTIFY {
            let id = captures
                .first()
                .and_then(Value::as_int)
                .and_then(|id| usize::try_from(id).ok())?;
            if id > 0 && id <= self.async_runtime.sources.len() {
                self.async_runtime.notifications.push_back(id);
            }
            return Some(Ok(Value::NULL));
        }
        if function != UNSUBSCRIBE {
            return None;
        }
        let id = captures
            .first()
            .and_then(Value::as_int)
            .and_then(|id| usize::try_from(id).ok())?;
        let subscription = captures.get(1).and_then(Value::as_int)?;
        if let Some(source) = id
            .checked_sub(1)
            .and_then(|index| self.async_runtime.sources.get_mut(index))
        {
            source.subscribers.retain(|item| item.id != subscription);
        }
        Some(Ok(Value::NULL))
    }

    fn finish_source(
        &mut self,
        id: usize,
        result: Result<Value, Value>,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<(), VmError> {
        let source = &mut self.async_runtime.sources[id - 1];
        if source.result.is_some() {
            return Ok(());
        }
        source.cancelled = source.cancel_called
            && result.as_ref().err().is_some_and(|failure| {
                failure.class_id().is_some_and(|class| {
                    is_instance_of_name(self.program, class, "CancellationException")
                })
            });
        source.result = Some(result);
        self.async_runtime.ready.retain(|ready| *ready != id);
        self.async_runtime
            .timers
            .retain(|Reverse((_, pending))| *pending != id);
        let subscribers = std::mem::take(&mut source.subscribers);
        if let Some((target, cancellation)) = self.async_runtime.root_wait
            && self.async_runtime.root_winner.is_none()
            && (target == id || cancellation == Some(id))
        {
            self.async_runtime.root_winner = Some(target == id);
        }
        for waiter in 1..=self.async_runtime.sources.len() {
            let source = &mut self.async_runtime.sources[waiter - 1];
            if let Some(Wait::Await {
                target,
                cancellation,
                winner,
                ..
            }) = &mut source.waiting
                && winner.is_none()
                && (*target == id || *cancellation == Some(id))
            {
                *winner = Some(*target == id);
                if !self.async_runtime.ready.contains(&waiter) {
                    self.async_runtime.ready.push_back(waiter);
                }
            }
        }
        for subscriber in subscribers {
            self.call_async_callback(&subscriber.callback, depth, calling_function, instruction)?;
        }
        Ok(())
    }

    fn cancel_source(
        &mut self,
        id: usize,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<(), VmError> {
        let source = &mut self.async_runtime.sources[id - 1];
        if source.result.is_some() || source.cancel_called {
            return Ok(());
        }
        source.cancel_called = true;
        if matches!(source.kind, SourceKind::Coroutine { .. }) && source.started {
            source.cancel_requested = true;
            if matches!(source.waiting, Some(Wait::Delay { .. })) {
                self.async_runtime
                    .timers
                    .retain(|Reverse((_, pending))| *pending != id);
            }
            if source.waiting.is_some()
                && !matches!(
                    source.waiting,
                    Some(
                        Wait::File { .. }
                            | Wait::FileRetry { .. }
                            | Wait::Open { .. }
                            | Wait::Info { .. }
                            | Wait::Directory { .. }
                            | Wait::RecursiveDirectory { .. }
                    )
                )
                && !self.async_runtime.ready.contains(&id)
            {
                self.async_runtime.ready.push_back(id);
            }
            return Ok(());
        }
        let error = self.native_exception(
            "CancellationException",
            b"operation cancelled".to_vec(),
            None,
            0,
            instruction.span,
        );
        let VmError::Thrown { value, .. } = error else {
            return Err(error);
        };
        self.finish_source(id, Err(value), depth, calling_function, instruction)
    }

    fn fire_timers(
        &mut self,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<(), VmError> {
        self.process_notifications(depth, calling_function, instruction)?;
        for id in 1..=self.async_runtime.sources.len() {
            let ready = match &mut self.async_runtime.sources[id - 1].waiting {
                Some(Wait::File { pending, .. }) => pending.poll(),
                Some(Wait::Open { pending, .. }) => pending.poll(),
                Some(Wait::Info { pending, .. }) => pending.poll(),
                Some(Wait::Directory { pending, .. }) => pending.poll(),
                Some(Wait::RecursiveDirectory { pending, .. }) => pending.poll(),
                Some(Wait::WorkerRetry { retry_at, .. }) => Instant::now() >= *retry_at,
                Some(Wait::FileRetry {
                    handle, retry_at, ..
                }) => {
                    !handle
                        .stream_file_busy()
                        .map_err(|kind| runtime(kind, instruction.span))?
                        && retry_at.is_none_or(|deadline| Instant::now() >= deadline)
                }
                _ => false,
            };
            if ready && !self.async_runtime.ready.contains(&id) {
                self.async_runtime.ready.push_back(id);
            }
        }
        let now = Instant::now();
        while let Some(Reverse((deadline, id))) = self.async_runtime.timers.peek().copied() {
            if deadline > now {
                break;
            }
            self.async_runtime.timers.pop();
            if self.async_runtime.sources[id - 1].result.is_some() {
                continue;
            }
            if matches!(self.async_runtime.sources[id - 1].kind, SourceKind::Timeout) {
                let error = self.native_exception(
                    "TimeoutException",
                    b"timeout expired".to_vec(),
                    None,
                    0,
                    instruction.span,
                );
                let VmError::Thrown { value, .. } = error else {
                    return Err(error);
                };
                self.finish_source(id, Err(value), depth, calling_function, instruction)?;
            } else if matches!(self.async_runtime.sources[id - 1].waiting,
                Some(Wait::Delay { deadline: waiting, .. }) if waiting == deadline)
                && !self.async_runtime.ready.contains(&id)
            {
                self.async_runtime.ready.push_back(id);
            }
        }
        Ok(())
    }

    fn next_wakeup_deadline(&self) -> Option<Instant> {
        let timer = self
            .async_runtime
            .timers
            .peek()
            .map(|Reverse((deadline, _))| *deadline);
        let file = self.async_runtime.sources.iter().any(|source| {
            matches!(
                source.waiting,
                Some(
                    Wait::File { .. }
                        | Wait::FileRetry { .. }
                        | Wait::Open { .. }
                        | Wait::Info { .. }
                        | Wait::Directory { .. }
                        | Wait::RecursiveDirectory { .. }
                        | Wait::WorkerRetry { .. }
                )
            )
        });
        if file {
            Some(
                timer.map_or(Instant::now() + Duration::from_millis(1), |deadline| {
                    deadline.min(Instant::now() + Duration::from_millis(1))
                }),
            )
        } else {
            timer
        }
    }

    fn run_ready(
        &mut self,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<bool, VmError> {
        self.fire_timers(depth, calling_function, instruction)?;
        if self.async_runtime.ready.is_empty() {
            return Ok(false);
        }
        let id = match &self.async_runtime.registration {
            Some(Registration::Custom(scheduler)) => {
                let scheduler = scheduler.clone();
                let ready = self
                    .async_runtime
                    .ready
                    .iter()
                    .map(|id| id_integer(*id))
                    .collect();
                let ready = Value::try_vector(Type::Int, ready)
                    .map_err(|kind| runtime(kind, instruction.span))?;
                let selected = self
                    .call_async_method(
                        &scheduler,
                        "chooseNext",
                        vec![ready],
                        depth,
                        calling_function,
                        instruction,
                    )
                    .map_err(|error| self.fatal_async_error(error))?;
                let Some(id) = selected.as_int().and_then(|id| usize::try_from(id).ok()) else {
                    return Err(self.fatal_async_error(self.async_error(
                        "scheduler returned an invalid coroutine ID",
                        instruction.span,
                    )));
                };
                let Some(position) = self
                    .async_runtime
                    .ready
                    .iter()
                    .position(|ready| *ready == id)
                else {
                    return Err(self.fatal_async_error(self.async_error(
                        "scheduler selected an ID that is not ready",
                        instruction.span,
                    )));
                };
                self.async_runtime
                    .ready
                    .remove(position)
                    .expect("ready index exists")
            }
            _ => self
                .async_runtime
                .ready
                .pop_front()
                .expect("ready queue is nonempty"),
        };
        if self.async_runtime.sources[id - 1].result.is_some() {
            return Ok(true);
        }
        let previous = self.async_runtime.current.replace(id);
        let saved_stack = std::mem::take(&mut self.call_stack);
        let saved_functions = std::mem::take(&mut self.active_functions);
        let mut frames = std::mem::take(&mut self.async_runtime.sources[id - 1].frames);
        if !self.async_runtime.sources[id - 1].started {
            let (callback, arguments) = match &mut self.async_runtime.sources[id - 1].kind {
                SourceKind::Coroutine {
                    callback,
                    arguments,
                } => (callback.take(), std::mem::take(arguments)),
                SourceKind::Timeout | SourceKind::External { .. } => {
                    unreachable!("ready source is a coroutine")
                }
            };
            let callback = callback.expect("queued coroutine has a callable");
            let (function, captures) = callback
                .closure_parts()
                .ok_or_else(|| self.async_error("async callable is invalid", instruction.span))?;
            let mut values = captures.to_vec();
            values.extend(arguments);
            self.async_runtime.sources[id - 1].started = true;
            if function == UNSUBSCRIBE || function == NOTIFY {
                let result = self
                    .invoke_async_callback(function, captures)
                    .expect("internal Async callable");
                self.call_stack = saved_stack;
                self.active_functions = saved_functions;
                self.async_runtime.current = previous;
                self.finish_source(id, Ok(result?), depth, calling_function, instruction)?;
                return Ok(true);
            }
            if self.program.functions[function.0 as usize].generator {
                let result = self.invoke_callee(
                    Callee::Function(function),
                    values,
                    0,
                    None,
                    calling_function,
                    instruction,
                );
                self.call_stack = saved_stack;
                self.active_functions = saved_functions;
                self.async_runtime.current = previous;
                match result {
                    Ok(value) => {
                        self.finish_source(id, Ok(value), depth, calling_function, instruction)?;
                    }
                    Err(VmError::Thrown { value, trace, .. }) => {
                        self.async_runtime.sources[id - 1].failure_trace = trace;
                        self.finish_source(id, Err(value), depth, calling_function, instruction)?;
                    }
                    Err(error) => return Err(error),
                }
                return Ok(true);
            }
            frames.push(TaskFrame {
                function,
                frame: self.new_frame(function, values, None),
                destination: None,
                caller_span: instruction.span,
                validate_countable: false,
                return_override: None,
                reflection_return: None,
                generator: None,
                collection: None,
                iterator: None,
                native_iterators: Vec::new(),
            });
        }
        self.async_runtime.sources[id - 1].suspended = false;
        let mut waiting = self.async_runtime.sources[id - 1].waiting.take();
        let retry_file = matches!(
            waiting,
            Some(Wait::FileRetry { .. } | Wait::WorkerRetry { .. })
        );
        let resume = if self.async_runtime.sources[id - 1].cancel_requested {
            self.async_runtime.sources[id - 1].cancel_requested = false;
            Some(Err(self.native_exception(
                "CancellationException",
                b"coroutine cancelled".to_vec(),
                None,
                0,
                waiting.as_ref().map_or(instruction.span, Wait::span),
            )))
        } else {
            waiting.as_mut().map(|wait| match wait {
                Wait::Yield { .. }
                | Wait::Delay { .. }
                | Wait::FileRetry { .. }
                | Wait::WorkerRetry { .. } => Ok(Value::NULL),
                Wait::Await {
                    target,
                    cancellation,
                    winner,
                    span,
                    ..
                } => {
                    if *winner == Some(true) {
                        self.read_result(*target, *span)
                    } else if *winner == Some(false) {
                        let cause = self.cancellation_cause(*cancellation);
                        let error = self.native_exception(
                            "WaitCancelledException",
                            b"wait cancelled".to_vec(),
                            None,
                            0,
                            *span,
                        );
                        if let VmError::Thrown { value, .. } = &error
                            && let Some(cause) = cause
                            && let Err(kind) = value.append_previous(cause)
                        {
                            return Err(runtime(kind, *span));
                        }
                        Err(error)
                    } else {
                        Err(self.async_error("coroutine resumed without a wakeup", *span))
                    }
                }
                Wait::File {
                    pending,
                    completion,
                    span,
                    ..
                } => self.file_result_value(
                    completion,
                    pending.take_result().expect("file result is ready"),
                    *span,
                ),
                Wait::Open {
                    pending,
                    completion,
                    span,
                    ..
                } => self.file_open_result_value(
                    completion,
                    pending.take_result().expect("file open result is ready"),
                    *span,
                ),
                Wait::Info { pending, span, .. } => self.file_info_value(
                    pending.take_result().expect("file info result is ready"),
                    *span,
                ),
                Wait::Directory {
                    pending,
                    completion,
                    span,
                    ..
                } => self.directory_result_value(
                    completion,
                    pending.take_result().expect("directory result is ready"),
                    *span,
                ),
                Wait::RecursiveDirectory {
                    pending,
                    completion,
                    span,
                    ..
                } => self.recursive_directory_result_value(
                    completion,
                    pending
                        .take_result()
                        .expect("recursive directory result is ready"),
                    *span,
                ),
            })
        };
        let result = if retry_file && resume.as_ref().is_some_and(Result::is_ok) {
            self.step_task(&mut frames)
        } else if let Some(resume) = resume {
            match resume {
                Ok(value) => {
                    let wait = waiting.as_ref().expect("resume has a wait");
                    let frame = &mut frames
                        .last_mut()
                        .expect("suspended coroutine has a frame")
                        .frame;
                    if let Some(destination) = wait.destination() {
                        frame.registers[destination.0 as usize] = Some(value);
                    }
                    frame.next_instruction += 1;
                    self.step_task(&mut frames)
                }
                Err(error) => self
                    .task_error(&mut frames, error)
                    .and_then(|()| self.step_task(&mut frames)),
            }
        } else {
            self.step_task(&mut frames)
        };
        self.call_stack = saved_stack;
        self.active_functions = saved_functions;
        self.async_runtime.current = previous;
        match result {
            Ok(TaskOutcome::Completed(value)) => {
                self.finish_source(id, Ok(value), depth, calling_function, instruction)?;
            }
            Ok(TaskOutcome::Suspended(wait)) => {
                let requeue = matches!(wait, Wait::Yield { .. });
                let source = &mut self.async_runtime.sources[id - 1];
                source.frames = frames;
                source.waiting = Some(wait);
                source.suspended = true;
                if let Some(Wait::Delay { deadline, .. }) = &source.waiting {
                    self.async_runtime.timers.push(Reverse((*deadline, id)));
                }
                if requeue {
                    self.async_runtime.ready.push_back(id);
                }
                let wake = match &self.async_runtime.sources[id - 1].waiting {
                    Some(Wait::Await {
                        target,
                        cancellation,
                        ..
                    }) if self.async_runtime.sources[*target - 1].result.is_some() => Some(true),
                    Some(Wait::Await {
                        cancellation: Some(cancel),
                        ..
                    }) if self.async_runtime.sources[*cancel - 1].result.is_some() => Some(false),
                    _ => None,
                };
                if let Some(winner) = wake {
                    if let Some(Wait::Await {
                        winner: selected, ..
                    }) = &mut self.async_runtime.sources[id - 1].waiting
                    {
                        *selected = Some(winner);
                    }
                    self.async_runtime.ready.push_back(id);
                }
            }
            Err(VmError::Thrown { value, trace, .. }) => {
                self.async_runtime.sources[id - 1].failure_trace = trace;
                self.finish_source(id, Err(value), depth, calling_function, instruction)?;
            }
            Err(error) => return Err(error),
        }
        if let Some(Wait::Await {
            target,
            cancellation,
            ..
        }) = &waiting
        {
            self.release_external_if_unused(*target, depth, calling_function, instruction)?;
            if let Some(cancellation) = cancellation {
                self.release_external_if_unused(
                    *cancellation,
                    depth,
                    calling_function,
                    instruction,
                )?;
            }
        }
        Ok(true)
    }

    fn call_async_method(
        &mut self,
        receiver: &Value,
        name: &str,
        arguments: Vec<Value>,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        let class_id = receiver
            .class_id()
            .ok_or_else(|| self.async_error("Async source must be an object", instruction.span))?;
        let class = &self.program.classes[class_id.0 as usize];
        let method = class
            .methods
            .iter()
            .find(|method| method.name == name)
            .ok_or_else(|| {
                self.async_error(
                    "Async source does not implement its contract",
                    instruction.span,
                )
            })?;
        let (callee, _) = self.dispatch_target(class_id, method.slot, instruction.span)?;
        let mut values = vec![receiver.clone()];
        values.extend(arguments);
        let previous = self.async_runtime.in_callback;
        self.async_runtime.in_callback = true;
        let result = self.invoke_callee(
            callee,
            values,
            depth,
            Some(class_id),
            calling_function,
            instruction,
        );
        self.async_runtime.in_callback = previous;
        result
    }

    fn task_call(
        &mut self,
        frames: &mut Vec<TaskFrame>,
        callee: Callee,
        arguments: Vec<Value>,
        called_class: Option<ClassId>,
        instruction: &Instruction,
    ) -> Result<CallOutcome, VmError> {
        let span = instruction.span;
        let destination = instruction.destination;
        let depth = frames.len();
        let calling_function =
            &self.program.functions[frames.last().expect("task frame").function.0 as usize];
        if let Callee::Builtin(
            builtin
            @ (Builtin::IteratorRewind | Builtin::IteratorAdvance | Builtin::GeneratorClose),
        ) = callee
            && let Some(outcome) =
                self.task_generator_resume(frames, builtin, &arguments[0], instruction)?
        {
            return Ok(outcome);
        }
        if let Callee::Builtin(builtin @ (Builtin::IteratorRewind | Builtin::IteratorAdvance)) =
            callee
            && let Some(class) = arguments[0].class_id()
            && is_instance_of_name(self.program, class, "RecursiveIteratorIterator")
        {
            let wrapper = &arguments[0];
            let walk_frames = if builtin == Builtin::IteratorRewind {
                wrapper
                    .set_property(PropertyId(7), Value::bool(false))
                    .map_err(|kind| runtime(kind, span))?;
                wrapper
                    .set_property(PropertyId(6), Value::integer(-1))
                    .map_err(|kind| runtime(kind, span))?;
                Self::store_walk_stack(wrapper, Vec::new(), span)?;
                wrapper
                    .set_property(PropertyId(9), Value::bool(false))
                    .map_err(|kind| runtime(kind, span))?;
                Vec::new()
            } else {
                let ready = wrapper
                    .property(PropertyId(7))
                    .map_err(|kind| runtime(kind, span))?
                    .as_bool()
                    == Some(true);
                if !ready {
                    let frame = &mut frames.last_mut().expect("walk caller").frame;
                    if let Some(destination) = destination {
                        frame.registers[destination.0 as usize] = Some(Value::NULL);
                    }
                    frame.next_instruction += 1;
                    return Ok(CallOutcome::Continue);
                }
                wrapper
                    .property(PropertyId(3))
                    .map_err(|kind| runtime(kind, span))?
                    .vector_values()
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?
                    .iter()
                    .map(|value| WalkFrame::decode(value, span))
                    .collect::<Result<Vec<_>, _>>()?
            };
            let property_int = |slot| -> Result<i64, VmError> {
                wrapper
                    .property(PropertyId(slot))
                    .map_err(|kind| runtime(kind, span))?
                    .as_int()
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))
            };
            let entry = frames.last_mut().expect("walk caller");
            let scratch =
                Register(u32::try_from(entry.frame.registers.len()).expect("register count"));
            entry.frame.registers.push(None);
            let nested = entry.iterator.is_some() || !entry.native_iterators.is_empty();
            entry
                .native_iterators
                .push(NativeIteratorContinuation::Walk(WalkContinuation {
                    wrapper: wrapper.clone(),
                    frames: walk_frames,
                    mode: property_int(1)?,
                    max_depth: property_int(2)?,
                    phase: if builtin == Builtin::IteratorRewind {
                        WalkPhase::BeginIteration
                    } else {
                        WalkPhase::Seek
                    },
                    key: None,
                    scratch,
                    instruction: instruction.clone(),
                    pending: false,
                    restore_instruction: false,
                    nested,
                }));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(Builtin::Count) = callee
            && let Some(class) = arguments[0].class_id()
            && is_instance_of_name(self.program, class, "Countable")
        {
            let method = self.program.classes[class.0 as usize]
                .methods
                .iter()
                .find(|method| method.name == "count")
                .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
            let (target, _) = self.dispatch_target(class, method.slot, span)?;
            return self.task_call(frames, target, arguments, Some(class), instruction);
        }
        if let Callee::Builtin(Builtin::LimitSeek) = callee {
            let position = arguments[1].as_int().expect("verified seek offset");
            let limit = arguments[0]
                .property(PropertyId(2))
                .map_err(|kind| runtime(kind, span))?
                .as_int()
                .expect("initialized limit");
            if position < 0 || limit >= 0 && position >= limit {
                return Err(self.native_exception(
                    "OutOfBoundsException",
                    b"limit iterator position is out of bounds".to_vec(),
                    None,
                    0,
                    span,
                ));
            }
            let entry = frames.last_mut().expect("limit seek caller");
            let scratch =
                Register(u32::try_from(entry.frame.registers.len()).expect("register count"));
            entry.frame.registers.push(None);
            let nested = entry.iterator.is_some() || !entry.native_iterators.is_empty();
            entry
                .native_iterators
                .push(NativeIteratorContinuation::LimitSeek(
                    LimitSeekContinuation {
                        wrapper: arguments[0].clone(),
                        position,
                        index: 0,
                        phase: LimitSeekPhase::Rewind,
                        scratch,
                        instruction: instruction.clone(),
                        pending: false,
                        restore_instruction: false,
                        nested,
                    },
                ));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(Builtin::CallbackFilterAccept | Builtin::RecursiveCallbackAccept) =
            callee
        {
            let callback = arguments[0]
                .property(PropertyId(2))
                .map_err(|kind| runtime(kind, span))?;
            let (target, captures) = callback
                .closure_parts()
                .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
            let mut values = captures.to_vec();
            values.extend(arguments[1..].iter().cloned());
            return self.task_call(frames, Callee::Function(target), values, None, instruction);
        }
        if let Callee::Builtin(Builtin::CachingToString) = callee
            && arguments[0]
                .property(PropertyId(6))
                .map_err(|kind| runtime(kind, span))?
                .as_bool()
                == Some(true)
        {
            let flags = arguments[0]
                .property(PropertyId(1))
                .map_err(|kind| runtime(kind, span))?
                .as_int()
                .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
            let slot = if flags & 4 != 0 {
                2
            } else if flags & 16 != 0 {
                0
            } else {
                3
            };
            let candidate = arguments[0]
                .property(PropertyId(slot))
                .map_err(|kind| runtime(kind, span))?;
            if candidate.output_bytes().is_none()
                && let Some(class) = candidate.class_id()
                && is_instance_of_name(self.program, class, "Stringable")
            {
                let method = self.program.classes[class.0 as usize]
                    .methods
                    .iter()
                    .find(|method| method.name == "__toString")
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
                let (target, _) = self.dispatch_target(class, method.slot, span)?;
                return self.task_call(frames, target, vec![candidate], Some(class), instruction);
            }
        }
        if let Callee::Builtin(builtin @ (Builtin::IteratorRewind | Builtin::IteratorAdvance)) =
            callee
            && let Some(class) = arguments[0].class_id()
            && is_instance_of_name(self.program, class, "CachingIterator")
        {
            let wrapper = &arguments[0];
            let inner = wrapper
                .property(PropertyId(0))
                .map_err(|kind| runtime(kind, span))?;
            if builtin == Builtin::IteratorAdvance {
                let current = wrapper
                    .property(PropertyId(6))
                    .map_err(|kind| runtime(kind, span))?
                    .as_bool()
                    == Some(true);
                let next = wrapper
                    .property(PropertyId(7))
                    .map_err(|kind| runtime(kind, span))?
                    .as_bool()
                    == Some(true);
                if !current || !next {
                    wrapper
                        .set_property(PropertyId(6), Value::bool(false))
                        .map_err(|kind| runtime(kind, span))?;
                    let frame = &mut frames.last_mut().expect("caching caller").frame;
                    if let Some(destination) = destination {
                        frame.registers[destination.0 as usize] = Some(Value::NULL);
                    }
                    frame.next_instruction += 1;
                    return Ok(CallOutcome::Continue);
                }
                for (from, to) in [(4, 2), (5, 3)] {
                    let value = wrapper
                        .property(PropertyId(from))
                        .map_err(|kind| runtime(kind, span))?;
                    wrapper
                        .set_property(PropertyId(to), value)
                        .map_err(|kind| runtime(kind, span))?;
                }
            }
            let entry = frames.last_mut().expect("caching caller");
            let scratch =
                Register(u32::try_from(entry.frame.registers.len()).expect("register count"));
            entry.frame.registers.push(None);
            let nested = entry.iterator.is_some() || !entry.native_iterators.is_empty();
            entry
                .native_iterators
                .push(NativeIteratorContinuation::Caching(CachingContinuation {
                    wrapper: wrapper.clone(),
                    inner,
                    phase: if builtin == Builtin::IteratorRewind {
                        CachingPhase::Rewind
                    } else {
                        CachingPhase::PullValid
                    },
                    pull: u32::from(builtin != Builtin::IteratorRewind),
                    key: None,
                    value: None,
                    scratch,
                    instruction: instruction.clone(),
                    pending: false,
                    restore_instruction: false,
                    nested,
                }));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(
            builtin @ (Builtin::IteratorRewind
            | Builtin::IteratorValid
            | Builtin::IteratorKey
            | Builtin::IteratorValue
            | Builtin::IteratorAdvance),
        ) = callee
            && arguments[0].class_id().is_some_and(|class| {
                matches!(
                    self.program.classes[class.0 as usize].name.as_str(),
                    "RegexIterator" | "RecursiveRegexIterator"
                )
            })
        {
            let wrapper = &arguments[0];
            let ready = wrapper
                .property(PropertyId(6))
                .map_err(|kind| runtime(kind, span))?
                .as_bool()
                == Some(true);
            if ready && matches!(builtin, Builtin::IteratorValid | Builtin::IteratorValue) {
                let value = if builtin == Builtin::IteratorValid {
                    Value::bool(true)
                } else {
                    wrapper
                        .property(PropertyId(7))
                        .map_err(|kind| runtime(kind, span))?
                };
                let frame = &mut frames.last_mut().expect("regex caller").frame;
                if let Some(destination) = destination {
                    frame.registers[destination.0 as usize] = Some(value);
                }
                frame.next_instruction += 1;
                return Ok(CallOutcome::Continue);
            }
            let (regex, options) = if !ready
                && matches!(
                    builtin,
                    Builtin::IteratorValid | Builtin::IteratorKey | Builtin::IteratorValue
                ) {
                let pattern = wrapper
                    .property(PropertyId(1))
                    .map_err(|kind| runtime(kind, span))?;
                let regex = pcre::compile(pattern.as_bytes().expect("verified pattern"))
                    .map_err(|message| runtime(RuntimeErrorKind::TypeError(message), span))?;
                let property_int = |slot| -> Result<i64, VmError> {
                    wrapper
                        .property(PropertyId(slot))
                        .map_err(|kind| runtime(kind, span))?
                        .as_int()
                        .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))
                };
                let options = RegexScanOptions {
                    mode: property_int(2)?,
                    invert: wrapper
                        .property(PropertyId(3))
                        .map_err(|kind| runtime(kind, span))?
                        .as_bool()
                        .expect("verified invert"),
                    flags: property_int(4)?,
                    replacement: wrapper
                        .property(PropertyId(5))
                        .map_err(|kind| runtime(kind, span))?,
                    recursive: wrapper.class_id().is_some_and(|class| {
                        self.program.classes[class.0 as usize].name == "RecursiveRegexIterator"
                    }),
                };
                (Some(regex), Some(options))
            } else {
                (None, None)
            };
            let inner = wrapper
                .property(PropertyId(0))
                .map_err(|kind| runtime(kind, span))?;
            let entry = frames.last_mut().expect("regex caller");
            let scratch =
                Register(u32::try_from(entry.frame.registers.len()).expect("register count"));
            entry.frame.registers.push(None);
            let nested = entry.iterator.is_some() || !entry.native_iterators.is_empty();
            entry
                .native_iterators
                .push(NativeIteratorContinuation::Regex(RegexContinuation {
                    builtin,
                    wrapper: wrapper.clone(),
                    inner,
                    regex,
                    options,
                    phase: match builtin {
                        Builtin::IteratorRewind => RegexPhase::Rewind,
                        Builtin::IteratorAdvance => RegexPhase::CheckForAdvance,
                        Builtin::IteratorKey if ready => RegexPhase::FinalKey,
                        _ => RegexPhase::CheckValid,
                    },
                    key: None,
                    entry: None,
                    value: None,
                    child: None,
                    matched: None,
                    scratch,
                    instruction: instruction.clone(),
                    pending: false,
                    restore_instruction: false,
                    nested,
                }));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(
            builtin @ (Builtin::IteratorRewind
            | Builtin::IteratorValid
            | Builtin::IteratorKey
            | Builtin::IteratorValue
            | Builtin::IteratorAdvance),
        ) = callee
            && arguments[0].class_id().is_some_and(|class| {
                self.program.classes[class.0 as usize].name == "InfiniteIterator"
            })
        {
            let wrapper = &arguments[0];
            let inner = wrapper
                .property(PropertyId(1))
                .map_err(|kind| runtime(kind, span))?;
            let exhausted = wrapper
                .property(PropertyId(2))
                .map_err(|kind| runtime(kind, span))?
                .as_bool()
                == Some(true);
            if builtin == Builtin::IteratorRewind {
                wrapper
                    .set_property(PropertyId(2), Value::bool(false))
                    .map_err(|kind| runtime(kind, span))?;
            } else if exhausted || inner.is_null() {
                let value = match builtin {
                    Builtin::IteratorValid => Value::bool(false),
                    Builtin::IteratorAdvance => Value::NULL,
                    _ => {
                        return Err(runtime(
                            RuntimeErrorKind::Bounds("iterator is exhausted".to_owned()),
                            span,
                        ));
                    }
                };
                let frame = &mut frames.last_mut().expect("infinite caller").frame;
                if let Some(destination) = destination {
                    frame.registers[destination.0 as usize] = Some(value);
                }
                frame.next_instruction += 1;
                return Ok(CallOutcome::Continue);
            }
            let entry = frames.last_mut().expect("infinite caller");
            let scratch =
                Register(u32::try_from(entry.frame.registers.len()).expect("register count"));
            entry.frame.registers.push(None);
            let nested = entry.iterator.is_some() || !entry.native_iterators.is_empty();
            entry
                .native_iterators
                .push(NativeIteratorContinuation::Infinite(InfiniteContinuation {
                    builtin,
                    wrapper: wrapper.clone(),
                    inner: (!inner.is_null()).then_some(inner),
                    fetched: None,
                    aggregate_depth: 0,
                    phase: if builtin == Builtin::IteratorRewind {
                        InfinitePhase::FetchSource
                    } else {
                        InfinitePhase::CheckInnerValid {
                            after_advance: false,
                        }
                    },
                    scratch,
                    instruction: instruction.clone(),
                    pending: false,
                    restore_instruction: false,
                    nested,
                }));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(
            builtin @ (Builtin::IteratorRewind
            | Builtin::IteratorValid
            | Builtin::IteratorKey
            | Builtin::IteratorValue
            | Builtin::IteratorAdvance),
        ) = callee
            && let Some(class) = arguments[0].class_id()
            && is_instance_of_name(self.program, class, "FilterIterator")
        {
            let wrapper = &arguments[0];
            let inner = wrapper
                .property(PropertyId(0))
                .map_err(|kind| runtime(kind, span))?;
            let ready = wrapper
                .property(PropertyId(1))
                .map_err(|kind| runtime(kind, span))?
                .as_bool()
                == Some(true);
            if builtin == Builtin::IteratorValid && ready {
                let frame = &mut frames.last_mut().expect("filter caller").frame;
                if let Some(destination) = destination {
                    frame.registers[destination.0 as usize] = Some(Value::bool(true));
                }
                frame.next_instruction += 1;
                return Ok(CallOutcome::Continue);
            }
            let phase = match builtin {
                Builtin::IteratorRewind => FilterPhase::Rewind,
                Builtin::IteratorAdvance => FilterPhase::CheckForAdvance,
                Builtin::IteratorValid | Builtin::IteratorKey | Builtin::IteratorValue if ready => {
                    FilterPhase::FinalValue
                }
                _ => FilterPhase::CheckValid,
            };
            let entry = frames.last_mut().expect("filter caller");
            let scratch =
                Register(u32::try_from(entry.frame.registers.len()).expect("register count"));
            entry.frame.registers.push(None);
            let nested = entry.iterator.is_some() || !entry.native_iterators.is_empty();
            entry
                .native_iterators
                .push(NativeIteratorContinuation::Filter(FilterContinuation {
                    builtin,
                    wrapper: wrapper.clone(),
                    inner,
                    kind: if is_instance_of_name(self.program, class, "RecursiveFilterIterator") {
                        FilterKind::Recursive
                    } else {
                        FilterKind::Plain
                    },
                    phase,
                    value: None,
                    key: None,
                    child: None,
                    scratch,
                    instruction: instruction.clone(),
                    pending: false,
                    restore_instruction: false,
                    nested,
                }));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(
            builtin @ (Builtin::IteratorRewind
            | Builtin::IteratorValid
            | Builtin::IteratorKey
            | Builtin::IteratorValue
            | Builtin::IteratorAdvance),
        ) = callee
            && arguments[0].class_id().is_some_and(|class| {
                self.program.classes[class.0 as usize].name == "AppendIterator"
            })
        {
            let wrapper = &arguments[0];
            let iterators = wrapper
                .property(PropertyId(0))
                .map_err(|kind| runtime(kind, span))?;
            let mut index = wrapper
                .property(PropertyId(1))
                .map_err(|kind| runtime(kind, span))?
                .as_int()
                .unwrap_or(-1);
            if builtin == Builtin::IteratorRewind {
                index = 0;
                wrapper
                    .set_property(PropertyId(1), Value::integer(0))
                    .map_err(|kind| runtime(kind, span))?;
            }
            let has_inner = usize::try_from(index)
                .ok()
                .and_then(|index| {
                    iterators
                        .vector_values()
                        .and_then(|values| values.get(index))
                })
                .is_some();
            if !has_inner {
                let value = match builtin {
                    Builtin::IteratorKey | Builtin::IteratorValue => {
                        return Err(runtime(
                            RuntimeErrorKind::Bounds("iterator is exhausted".to_owned()),
                            span,
                        ));
                    }
                    Builtin::IteratorValid => Value::bool(false),
                    _ => Value::NULL,
                };
                let frame = &mut frames.last_mut().expect("append caller").frame;
                if let Some(destination) = destination {
                    frame.registers[destination.0 as usize] = Some(value);
                }
                frame.next_instruction += 1;
                return Ok(CallOutcome::Continue);
            }
            let entry = frames.last_mut().expect("append caller");
            let scratch =
                Register(u32::try_from(entry.frame.registers.len()).expect("register count"));
            entry.frame.registers.push(None);
            let nested = entry.iterator.is_some() || !entry.native_iterators.is_empty();
            entry
                .native_iterators
                .push(NativeIteratorContinuation::Append(AppendContinuation {
                    builtin,
                    wrapper: wrapper.clone(),
                    iterators,
                    index,
                    phase: if builtin == Builtin::IteratorRewind {
                        AppendPhase::Rewind
                    } else {
                        AppendPhase::CheckValid {
                            after_advance: false,
                        }
                    },
                    scratch,
                    instruction: instruction.clone(),
                    pending: false,
                    restore_instruction: false,
                    nested,
                }));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(
            builtin @ (Builtin::IteratorRewind
            | Builtin::IteratorValid
            | Builtin::IteratorKey
            | Builtin::IteratorValue
            | Builtin::IteratorAdvance),
        ) = callee
            && arguments[0]
                .class_id()
                .is_some_and(|class| self.program.classes[class.0 as usize].name == "LimitIterator")
        {
            let wrapper = &arguments[0];
            let property_int = |slot| -> Result<i64, VmError> {
                wrapper
                    .property(PropertyId(slot))
                    .map_err(|kind| runtime(kind, span))?
                    .as_int()
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))
            };
            let position = property_int(3)?;
            let limit = property_int(2)?;
            let has_value = limit < 0 || position < limit;
            if !has_value && builtin != Builtin::IteratorRewind {
                let value = match builtin {
                    Builtin::IteratorValid => Value::bool(false),
                    Builtin::IteratorAdvance => Value::NULL,
                    _ => {
                        return Err(runtime(
                            RuntimeErrorKind::Bounds("iterator is exhausted".to_owned()),
                            span,
                        ));
                    }
                };
                let frame = &mut frames.last_mut().expect("limit caller").frame;
                if let Some(destination) = destination {
                    frame.registers[destination.0 as usize] = Some(value);
                }
                frame.next_instruction += 1;
                return Ok(CallOutcome::Continue);
            }
            let inner = wrapper
                .property(PropertyId(0))
                .map_err(|kind| runtime(kind, span))?;
            let phase = match builtin {
                Builtin::IteratorRewind => LimitPhase::Rewind,
                Builtin::IteratorValid => LimitPhase::Valid,
                _ => LimitPhase::Guard,
            };
            let entry = frames.last_mut().expect("limit caller");
            let scratch =
                Register(u32::try_from(entry.frame.registers.len()).expect("register count"));
            entry.frame.registers.push(None);
            let nested = entry.iterator.is_some() || !entry.native_iterators.is_empty();
            entry
                .native_iterators
                .push(NativeIteratorContinuation::Limit(LimitContinuation {
                    builtin,
                    wrapper: wrapper.clone(),
                    inner,
                    phase,
                    position,
                    remaining: property_int(1)?.max(0),
                    scratch,
                    instruction: instruction.clone(),
                    pending: false,
                    restore_instruction: false,
                    nested,
                }));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(
            builtin @ (Builtin::IteratorRewind
            | Builtin::IteratorValid
            | Builtin::IteratorKey
            | Builtin::IteratorValue
            | Builtin::IteratorAdvance),
        ) = callee
            && arguments[0].class_id().is_some_and(|class| {
                self.program.classes[class.0 as usize].name == "IteratorIterator"
            })
        {
            let mut inner = arguments[0].clone();
            let mut seen = Vec::new();
            loop {
                if seen.contains(&inner) {
                    return Err(self.async_error("iterator delegation cycle", span));
                }
                seen.push(inner.clone());
                self.tick(span)?;
                inner = inner
                    .property(PropertyId(0))
                    .map_err(|kind| runtime(kind, span))?;
                let class = inner
                    .class_id()
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
                if self.program.classes[class.0 as usize].name != "IteratorIterator" {
                    let method_name = match builtin {
                        Builtin::IteratorRewind => "rewind",
                        Builtin::IteratorValid => "valid",
                        Builtin::IteratorKey => "key",
                        Builtin::IteratorValue => "value",
                        Builtin::IteratorAdvance => "advance",
                        _ => unreachable!(),
                    };
                    let method = self.program.classes[class.0 as usize]
                        .methods
                        .iter()
                        .find(|method| method.name == method_name)
                        .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
                    let (target, _) = self.dispatch_target(class, method.slot, span)?;
                    return self.task_call(frames, target, vec![inner], Some(class), instruction);
                }
            }
        }
        if let Callee::Builtin(builtin) = callee
            && let Some((path, completion)) =
                self.file_open_request(builtin, &arguments, instruction)?
        {
            return match start_open_file(path, completion.flags)
                .map_err(|kind| runtime(kind, span))?
            {
                OpenFileStart::Pending(pending) => Ok(CallOutcome::Suspend(Wait::Open {
                    pending,
                    completion,
                    span,
                    destination,
                })),
                OpenFileStart::QueueFull => Ok(CallOutcome::Suspend(Wait::WorkerRetry {
                    span,
                    retry_at: Instant::now() + Duration::from_millis(1),
                })),
            };
        }
        if let Callee::Builtin(Builtin::FileInfo(operation)) = callee
            && file_info_uses_host(operation)
        {
            let path_value = arguments[0]
                .property(PropertyId(0))
                .map_err(|kind| runtime(kind, span))?;
            let original =
                std::str::from_utf8(path_value.as_bytes().expect("initialized file info path"))
                    .expect("constructor validates UTF-8");
            let path = Path::new(original);
            let resolved = if path.is_relative() {
                self.filesystem_base.join(path)
            } else {
                path.to_path_buf()
            };
            return match start_file_info(resolved, operation).map_err(|kind| runtime(kind, span))? {
                FileInfoStart::Pending(pending) => Ok(CallOutcome::Suspend(Wait::Info {
                    pending,
                    span,
                    destination,
                })),
                FileInfoStart::QueueFull => Ok(CallOutcome::Suspend(Wait::WorkerRetry {
                    span,
                    retry_at: Instant::now() + Duration::from_millis(1),
                })),
            };
        }
        if let Callee::Builtin(Builtin::Directory(operation)) = callee
            && let Some((path, completion)) =
                self.directory_request(operation, &arguments, instruction)
        {
            return match start_directory_read(path).map_err(|kind| runtime(kind, span))? {
                DirectoryStart::Pending(pending) => Ok(CallOutcome::Suspend(Wait::Directory {
                    pending,
                    completion: Box::new(completion),
                    span,
                    destination,
                })),
                DirectoryStart::QueueFull => Ok(CallOutcome::Suspend(Wait::WorkerRetry {
                    span,
                    retry_at: Instant::now() + Duration::from_millis(1),
                })),
            };
        }
        if let Callee::Builtin(Builtin::Directory(DirectoryBuiltin::RecursiveValue)) = callee
            && let Some((path, follow, completion)) =
                self.recursive_directory_request(&arguments[0], instruction)?
        {
            return match start_recursive_directory_read(path, follow)
                .map_err(|kind| runtime(kind, span))?
            {
                RecursiveDirectoryStart::Pending(pending) => {
                    Ok(CallOutcome::Suspend(Wait::RecursiveDirectory {
                        pending,
                        completion: Box::new(completion),
                        span,
                        destination,
                    }))
                }
                RecursiveDirectoryStart::QueueFull => Ok(CallOutcome::Suspend(Wait::WorkerRetry {
                    span,
                    retry_at: Instant::now() + Duration::from_millis(1),
                })),
            };
        }
        if let Callee::Builtin(Builtin::Reflection(
            operation @ (ReflectionBuiltin::FunctionInvokeArgs
            | ReflectionBuiltin::MethodInvokeArgs),
        )) = callee
        {
            let (target, bound, reflected_class, return_type) =
                self.prepare_reflection_invoke(operation, &arguments, span)?;
            if let Callee::Function(function) = target
                && !self.program.functions[function.0 as usize].generator
            {
                let before = frames.len();
                let result = self.task_call(frames, target, bound, reflected_class, instruction)?;
                if frames.len() > before {
                    frames
                        .last_mut()
                        .expect("reflected frame")
                        .reflection_return = Some(return_type);
                }
                return Ok(result);
            }
            if matches!(target, Callee::Builtin(_)) {
                return self.task_call(frames, target, bound, reflected_class, instruction);
            }
        }
        if let Callee::Builtin(Builtin::Reflection(ReflectionBuiltin::ClassNewInstanceArgs)) =
            callee
        {
            let (object, target, bound, reflected_class) =
                self.prepare_reflection_new_instance(&arguments, span)?;
            if let Callee::Function(function) = target
                && !self.program.functions[function.0 as usize].generator
            {
                let before = frames.len();
                let result =
                    self.task_call(frames, target, bound, Some(reflected_class), instruction)?;
                if frames.len() > before {
                    frames
                        .last_mut()
                        .expect("reflected constructor")
                        .return_override = Some(object);
                }
                return Ok(result);
            }
        }
        if let Callee::Builtin(
            builtin @ (Builtin::VectorMap
            | Builtin::VectorFilter
            | Builtin::MapTransform
            | Builtin::MapFilter),
        ) = callee
        {
            let (input, output) = match builtin {
                Builtin::VectorMap | Builtin::VectorFilter => {
                    let Type::Vector(element) =
                        instruction.ty.as_ref().expect("verified vector result")
                    else {
                        unreachable!()
                    };
                    (
                        CollectionInput::Vector(arguments[0].clone()),
                        Value::try_vector(element.as_ref().clone(), Vec::new())
                            .map_err(|kind| runtime(kind, span))?,
                    )
                }
                Builtin::MapTransform | Builtin::MapFilter => {
                    let Type::Map(key, value) =
                        instruction.ty.as_ref().expect("verified map result")
                    else {
                        unreachable!()
                    };
                    (
                        CollectionInput::Map(arguments[0].clone()),
                        Value::try_map(key.as_ref().clone(), value.as_ref().clone(), Vec::new())
                            .map_err(|kind| runtime(kind, span))?,
                    )
                }
                _ => unreachable!(),
            };
            frames.last_mut().expect("collection caller").collection =
                Some(Box::new(CollectionContinuation {
                    builtin,
                    callback: arguments[1].clone(),
                    input,
                    index: 0,
                    output,
                    instruction: instruction.clone(),
                    pending: false,
                }));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(
            builtin @ (Builtin::IteratorCount
            | Builtin::IteratorToVector
            | Builtin::IteratorToMap
            | Builtin::IteratorApply),
        ) = callee
        {
            let output = match (builtin, instruction.ty.as_ref()) {
                (Builtin::IteratorToVector, Some(Type::Vector(element))) => {
                    Value::try_vector(element.as_ref().clone(), Vec::new())
                }
                (Builtin::IteratorToMap, Some(Type::Map(key, value))) => {
                    Value::try_map(key.as_ref().clone(), value.as_ref().clone(), Vec::new())
                }
                (Builtin::IteratorCount | Builtin::IteratorApply, _) => Ok(Value::NULL),
                _ => unreachable!("verified iterator conversion"),
            }
            .map_err(|kind| runtime(kind, span))?;
            let frame = &mut frames.last_mut().expect("iterator caller").frame;
            let scratch = Register(u32::try_from(frame.registers.len()).expect("register count"));
            frame.registers.push(None);
            frames.last_mut().expect("iterator caller").iterator =
                Some(Box::new(IteratorContinuation {
                    builtin,
                    iterator: arguments[0].clone(),
                    callback: (builtin == Builtin::IteratorApply).then(|| arguments[1].clone()),
                    output,
                    count: 0,
                    key: None,
                    value: None,
                    stop: false,
                    phase: IteratorPhase::Valid,
                    scratch,
                    instruction: instruction.clone(),
                    pending: false,
                    restore_instruction: false,
                }));
            return Ok(CallOutcome::Continue);
        }
        if let Callee::Builtin(builtin) = callee {
            if let Builtin::FileObject(operation) = builtin
                && matches!(
                    operation,
                    FileObjectBuiltin::Fread
                        | FileObjectBuiltin::Fgets
                        | FileObjectBuiltin::Rewind
                        | FileObjectBuiltin::Advance
                        | FileObjectBuiltin::Seek
                        | FileObjectBuiltin::Eof
                )
            {
                let object = &arguments[0];
                let stream = object
                    .property(PropertyId(1))
                    .map_err(|kind| runtime(kind, span))?;
                if stream
                    .stream_file_busy()
                    .map_err(|kind| self.stream_exception(kind, span))?
                {
                    return Ok(CallOutcome::Suspend(Wait::FileRetry {
                        handle: stream,
                        span,
                        retry_at: None,
                    }));
                }
                let file_operation = match operation {
                    FileObjectBuiltin::Fread => {
                        StreamFileOperation::Read(self.nonnegative_stream_value(
                            arguments[1].as_int().expect("verified read length"),
                            "read length",
                            span,
                        )?)
                    }
                    FileObjectBuiltin::Fgets | FileObjectBuiltin::Advance => {
                        StreamFileOperation::ReadLine
                    }
                    FileObjectBuiltin::Rewind => StreamFileOperation::SeekLine(0),
                    FileObjectBuiltin::Eof => StreamFileOperation::Eof,
                    FileObjectBuiltin::Seek => {
                        StreamFileOperation::SeekLine(self.nonnegative_stream_value(
                            arguments[1].as_int().expect("verified line"),
                            "line number",
                            span,
                        )?)
                    }
                    _ => unreachable!(),
                };
                if operation != FileObjectBuiltin::Advance
                    || !object
                        .property(PropertyId(2))
                        .map_err(|kind| runtime(kind, span))?
                        .is_null()
                {
                    if operation == FileObjectBuiltin::Advance {
                        object
                            .property(PropertyId(3))
                            .map_err(|kind| runtime(kind, span))?
                            .as_int()
                            .expect("initialized line index")
                            .checked_add(1)
                            .ok_or_else(|| {
                                runtime(
                                    RuntimeErrorKind::Arithmetic("line index overflow".to_owned()),
                                    span,
                                )
                            })?;
                    }
                    match stream
                        .start_stream_file_operation(file_operation)
                        .map_err(|kind| self.stream_exception(kind, span))?
                    {
                        Some(StreamFileStart::Pending(pending)) => {
                            if operation == FileObjectBuiltin::Fgets {
                                object
                                    .set_property(PropertyId(2), Value::NULL)
                                    .map_err(|kind| runtime(kind, span))?;
                            }
                            return Ok(CallOutcome::Suspend(Wait::File {
                                pending,
                                completion: FileCompletion::FileObject {
                                    operation,
                                    object: object.clone(),
                                    line: (operation == FileObjectBuiltin::Seek)
                                        .then(|| arguments[1].as_int().expect("verified line")),
                                },
                                span,
                                destination,
                            }));
                        }
                        Some(StreamFileStart::Busy) => {
                            return Ok(CallOutcome::Suspend(Wait::FileRetry {
                                handle: stream,
                                span,
                                retry_at: None,
                            }));
                        }
                        Some(StreamFileStart::QueueFull) => {
                            return Ok(CallOutcome::Suspend(Wait::FileRetry {
                                handle: stream,
                                span,
                                retry_at: Some(Instant::now() + Duration::from_millis(1)),
                            }));
                        }
                        None => {}
                    }
                }
            }
            if let Some(operation) = self.file_operation(builtin, &arguments, span)? {
                match arguments[0]
                    .start_stream_file_operation(operation)
                    .map_err(|kind| self.stream_exception(kind, span))?
                {
                    Some(StreamFileStart::Busy) => {
                        return Ok(CallOutcome::Suspend(Wait::FileRetry {
                            handle: arguments[0].clone(),
                            span,
                            retry_at: None,
                        }));
                    }
                    Some(StreamFileStart::QueueFull) => {
                        return Ok(CallOutcome::Suspend(Wait::FileRetry {
                            handle: arguments[0].clone(),
                            span,
                            retry_at: Some(Instant::now() + Duration::from_millis(1)),
                        }));
                    }
                    Some(StreamFileStart::Pending(pending)) => {
                        return Ok(CallOutcome::Suspend(Wait::File {
                            pending,
                            completion: FileCompletion::Stream(builtin),
                            span,
                            destination,
                        }));
                    }
                    None => {}
                }
            } else if matches!(
                builtin,
                Builtin::StreamTell
                    | Builtin::StreamClose
                    | Builtin::StreamIsReadable
                    | Builtin::StreamIsWritable
                    | Builtin::StreamIsSeekable
                    | Builtin::StreamIsClosed
            ) && arguments[0]
                .stream_file_busy()
                .map_err(|kind| self.stream_exception(kind, span))?
            {
                return Ok(CallOutcome::Suspend(Wait::FileRetry {
                    handle: arguments[0].clone(),
                    span,
                    retry_at: None,
                }));
            }
        }
        match callee {
            Callee::Function(target) if target == UNSUBSCRIBE || target == NOTIFY => {
                let value = self
                    .invoke_async_callback(target, &arguments)
                    .expect("native callback");
                let value = value?;
                let frame = &mut frames.last_mut().expect("task frame").frame;
                if let Some(destination) = destination {
                    frame.registers[destination.0 as usize] = Some(value);
                }
                frame.next_instruction += 1;
                Ok(CallOutcome::Continue)
            }
            Callee::Function(target) if !self.program.functions[target.0 as usize].generator => {
                let target_function = &self.program.functions[target.0 as usize];
                let validate_countable = target_function.owner.is_some()
                    && !target_function.static_method
                    && target_function.name.ends_with("::count")
                    && arguments
                        .first()
                        .and_then(Value::class_id)
                        .is_some_and(|class| is_instance_of_name(self.program, class, "Countable"));
                if self
                    .limits
                    .max_stack_depth
                    .is_some_and(|limit| depth > limit)
                {
                    return Err(VmError::StackDepthLimit {
                        limit: self.limits.max_stack_depth.expect("limit exists"),
                        span,
                    });
                }
                frames
                    .last_mut()
                    .expect("task frame")
                    .frame
                    .next_instruction += 1;
                frames.push(TaskFrame {
                    function: target,
                    frame: self.new_frame(target, arguments, called_class),
                    destination,
                    caller_span: span,
                    validate_countable,
                    return_override: None,
                    reflection_return: None,
                    generator: None,
                    collection: None,
                    iterator: None,
                    native_iterators: Vec::new(),
                });
                Ok(CallOutcome::Continue)
            }
            Callee::Builtin(Builtin::Suspend | Builtin::Delay | Builtin::Await) => {
                let wait = match callee {
                    Callee::Builtin(Builtin::Suspend) => {
                        self.require_scheduler(span)?;
                        self.check_async_cancellation(span)?;
                        Wait::Yield { span, destination }
                    }
                    Callee::Builtin(Builtin::Delay) => {
                        self.require_scheduler(span)?;
                        self.check_async_cancellation(span)?;
                        let milliseconds = arguments[0].as_int().expect("verified delay duration");
                        if milliseconds < 0 {
                            return Err(self.async_error("delay duration cannot be negative", span));
                        }
                        if milliseconds == 0 {
                            Wait::Yield { span, destination }
                        } else {
                            let deadline = Instant::now()
                                .checked_add(Duration::from_millis(
                                    u64::try_from(milliseconds).expect("positive duration"),
                                ))
                                .ok_or_else(|| {
                                    self.async_error("delay duration is too large", span)
                                })?;
                            Wait::Delay {
                                deadline,
                                span,
                                destination,
                            }
                        }
                    }
                    Callee::Builtin(Builtin::Await) => {
                        let target = self.completion_id(
                            &arguments[0],
                            depth,
                            calling_function,
                            instruction,
                        )?;
                        if self.async_runtime.sources[target - 1].result.is_some() {
                            let value = self.read_result(target, span)?;
                            let frame = &mut frames.last_mut().expect("task frame").frame;
                            if let Some(destination) = destination {
                                frame.registers[destination.0 as usize] = Some(value);
                            }
                            frame.next_instruction += 1;
                            return Ok(CallOutcome::Continue);
                        }
                        self.require_scheduler(span)?;
                        self.check_async_cancellation(span)?;
                        let cancellation = if arguments[1].is_null() {
                            None
                        } else {
                            Some(self.completion_id(
                                &arguments[1],
                                depth,
                                calling_function,
                                instruction,
                            )?)
                        };
                        Wait::Await {
                            target,
                            cancellation,
                            winner: None,
                            span,
                            destination,
                        }
                    }
                    _ => unreachable!(),
                };
                Ok(CallOutcome::Suspend(wait))
            }
            other => {
                let value = self.invoke_callee(
                    other,
                    arguments,
                    depth,
                    called_class,
                    calling_function,
                    instruction,
                )?;
                let frame = &mut frames.last_mut().expect("task frame").frame;
                if let Some(destination) = destination {
                    frame.registers[destination.0 as usize] = Some(value);
                }
                frame.next_instruction += 1;
                Ok(CallOutcome::Continue)
            }
        }
    }

    fn advance_collection(&mut self, frames: &mut Vec<TaskFrame>) -> Result<(), VmError> {
        let entry = frames.last_mut().expect("collection frame");
        let mut state = entry.collection.take().expect("collection continuation");
        let span = state.instruction.span;
        if state.index == state.len() {
            if let Some(destination) = state.instruction.destination {
                entry.frame.registers[destination.0 as usize] = Some(state.output);
            }
            entry.frame.next_instruction += 1;
            return Ok(());
        }
        assert!(
            !state.pending,
            "callback frame must return before collection resumes"
        );
        self.tick(span)?;
        let callback_arguments = state.callback_arguments();
        let (target, captures) = state
            .callback
            .closure_parts()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        if let Some(result) = self.invoke_async_callback(target, captures) {
            state.receive(result?).map_err(|kind| runtime(kind, span))?;
            frames.last_mut().expect("collection frame").collection = Some(state);
            return Ok(());
        }
        let mut arguments = captures.to_vec();
        arguments.extend(callback_arguments);
        if self.program.functions[target.0 as usize].generator {
            let function = &self.program.functions[entry.function.0 as usize];
            let result = self.invoke_callee(
                Callee::Function(target),
                arguments,
                frames.len(),
                None,
                function,
                &state.instruction,
            )?;
            state.receive(result).map_err(|kind| runtime(kind, span))?;
            frames.last_mut().expect("collection frame").collection = Some(state);
            return Ok(());
        }
        let depth = frames.len();
        if let Some(limit) = self.limits.max_stack_depth
            && depth > limit
        {
            return Err(VmError::StackDepthLimit { limit, span });
        }
        state.pending = true;
        frames.last_mut().expect("collection frame").collection = Some(state);
        frames.push(TaskFrame {
            function: target,
            frame: self.new_frame(target, arguments, None),
            destination: None,
            caller_span: span,
            validate_countable: false,
            return_override: None,
            reflection_return: None,
            generator: None,
            collection: None,
            iterator: None,
            native_iterators: Vec::new(),
        });
        Ok(())
    }

    fn advance_iterator(&mut self, frames: &mut Vec<TaskFrame>) -> Result<CallOutcome, VmError> {
        let parent = frames.len() - 1;
        let mut state = frames[parent]
            .iterator
            .take()
            .expect("iterator continuation");
        let span = state.instruction.span;
        if state.pending {
            let frame = &mut frames[parent].frame;
            if state.restore_instruction {
                frame.next_instruction -= 1;
                state.restore_instruction = false;
            }
            let value = frame.registers[state.scratch.0 as usize]
                .take()
                .expect("iterator method result");
            state.pending = false;
            if matches!(state.phase, IteratorPhase::Valid) && value.as_bool() == Some(true) {
                self.tick(span)?;
            }
            if state.receive(value).map_err(|kind| runtime(kind, span))? {
                let result = if matches!(
                    state.builtin,
                    Builtin::IteratorCount | Builtin::IteratorApply
                ) {
                    Value::integer(state.count)
                } else {
                    state.output
                };
                let frame = &mut frames[parent].frame;
                frame.registers.pop();
                if let Some(destination) = state.instruction.destination {
                    frame.registers[destination.0 as usize] = Some(result);
                }
                frame.next_instruction += 1;
                return Ok(CallOutcome::Continue);
            }
            frames[parent].iterator = Some(state);
            return Ok(CallOutcome::Continue);
        }

        let (callee, arguments, called_class, ty) =
            if matches!(state.phase, IteratorPhase::Callback) {
                let callback = state.callback.as_ref().expect("iterator callback");
                let (target, captures) = callback
                    .closure_parts()
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
                let mut arguments = captures.to_vec();
                arguments.push(state.value.as_ref().expect("iterator value").clone());
                arguments.push(state.key.as_ref().expect("iterator key").clone());
                (Callee::Function(target), arguments, None, Type::Bool)
            } else {
                let name = match state.phase {
                    IteratorPhase::Valid => "valid",
                    IteratorPhase::Key => "key",
                    IteratorPhase::Value => "value",
                    IteratorPhase::Advance => "advance",
                    IteratorPhase::Callback => unreachable!(),
                };
                let class = state
                    .iterator
                    .class_id()
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
                let method = self.program.classes[class.0 as usize]
                    .methods
                    .iter()
                    .find(|method| method.name == name)
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
                let ty = substitute_type_arguments(
                    &method.return_type,
                    method.declaring_class,
                    state.iterator.type_arguments().unwrap_or_default(),
                );
                let (callee, _) = self.dispatch_target(class, method.slot, span)?;
                (callee, vec![state.iterator.clone()], Some(class), ty)
            };
        let mut instruction = state.instruction.clone();
        instruction.destination = Some(state.scratch);
        instruction.ty = Some(ty);
        frames[parent].iterator = Some(state);
        let native_depth = frames[parent].native_iterators.len();
        let result = self.task_call(frames, callee, arguments, called_class, &instruction)?;
        let entry = &mut frames[parent];
        let state = entry
            .iterator
            .as_mut()
            .expect("iterator continuation survives method call");
        match result {
            CallOutcome::Continue => {
                if entry.native_iterators.len() == native_depth {
                    entry.frame.next_instruction -= 1;
                }
                state.pending = true;
                Ok(CallOutcome::Continue)
            }
            CallOutcome::Suspend(wait) => {
                if !matches!(wait, Wait::FileRetry { .. } | Wait::WorkerRetry { .. }) {
                    state.pending = true;
                    state.restore_instruction = true;
                }
                Ok(CallOutcome::Suspend(wait))
            }
        }
    }

    fn advance_filter(&mut self, frames: &mut Vec<TaskFrame>) -> Result<CallOutcome, VmError> {
        let parent = frames.len() - 1;
        let NativeIteratorContinuation::Filter(mut state) = frames[parent]
            .native_iterators
            .pop()
            .expect("filter continuation")
        else {
            unreachable!("filter continuation is topmost");
        };
        let span = state.instruction.span;
        let finish = if state.pending {
            let frame = &mut frames[parent].frame;
            if state.restore_instruction {
                frame.next_instruction -= 1;
                state.restore_instruction = false;
            }
            let value = frame.registers[state.scratch.0 as usize]
                .take()
                .expect("filter method result");
            state.pending = false;
            match state.phase {
                FilterPhase::Rewind => {
                    state
                        .wrapper
                        .set_property(PropertyId(1), Value::bool(false))
                        .map_err(|kind| runtime(kind, span))?;
                    Some(Value::NULL)
                }
                FilterPhase::CheckForAdvance => {
                    if value.as_bool() == Some(true) {
                        state.phase = FilterPhase::Advance;
                        None
                    } else {
                        state
                            .wrapper
                            .set_property(PropertyId(1), Value::bool(false))
                            .map_err(|kind| runtime(kind, span))?;
                        Some(Value::NULL)
                    }
                }
                FilterPhase::CheckValid => {
                    if value.as_bool() == Some(true) {
                        self.tick(span)?;
                        state.phase = FilterPhase::ReadValue;
                        None
                    } else if state.builtin == Builtin::IteratorValid {
                        Some(Value::bool(false))
                    } else {
                        return Err(runtime(
                            RuntimeErrorKind::Bounds("iterator is exhausted".to_owned()),
                            span,
                        ));
                    }
                }
                FilterPhase::ReadValue => {
                    state.value = Some(value);
                    state.phase = FilterPhase::ReadKey;
                    None
                }
                FilterPhase::ReadKey => {
                    state.key = Some(value);
                    state.phase = FilterPhase::Accept;
                    None
                }
                FilterPhase::Accept | FilterPhase::AcceptParentChild => {
                    let accepted = if matches!(state.phase, FilterPhase::AcceptParentChild) {
                        !value.is_null()
                    } else {
                        value.as_bool() == Some(true)
                    };
                    if accepted {
                        state
                            .wrapper
                            .set_property(PropertyId(1), Value::bool(true))
                            .map_err(|kind| runtime(kind, span))?;
                        if state.builtin == Builtin::IteratorValid {
                            Some(Value::bool(true))
                        } else {
                            state.phase = FilterPhase::FinalValue;
                            None
                        }
                    } else {
                        state.phase = FilterPhase::SkipAdvance;
                        None
                    }
                }
                FilterPhase::SkipAdvance => {
                    state.phase = FilterPhase::CheckValid;
                    None
                }
                FilterPhase::FinalValue => {
                    if state.kind == FilterKind::Recursive
                        && state.builtin == Builtin::IteratorValue
                    {
                        state.value = Some(value);
                        state.phase = FilterPhase::ReadChild;
                        None
                    } else {
                        Some(value)
                    }
                }
                FilterPhase::ReadChild => {
                    if value.is_null() {
                        Some(state.value.take().expect("recursive filter entry"))
                    } else {
                        state.child = Some(self.recursive_filter_child_wrapper(
                            &state.wrapper,
                            value,
                            span,
                        )?);
                        state.phase = FilterPhase::ReadWrappedValue;
                        None
                    }
                }
                FilterPhase::ReadWrappedValue => Some(self.recursive_filter_node(
                    &state.wrapper,
                    value,
                    state.child.take().expect("recursive child wrapper"),
                    span,
                )?),
                FilterPhase::Advance => {
                    state
                        .wrapper
                        .set_property(PropertyId(1), Value::bool(false))
                        .map_err(|kind| runtime(kind, span))?;
                    Some(Value::NULL)
                }
            }
        } else {
            None
        };
        if let Some(value) = finish {
            let frame = &mut frames[parent].frame;
            frame.registers.pop();
            if let Some(destination) = state.instruction.destination {
                frame.registers[destination.0 as usize] = Some(value);
            }
            if !state.nested {
                frame.next_instruction += 1;
            }
            return Ok(CallOutcome::Continue);
        }
        if matches!(state.phase, FilterPhase::Accept) && state.kind == FilterKind::Recursive {
            let class = state.wrapper.class_id().expect("filter class");
            let method = self.program.classes[class.0 as usize]
                .methods
                .iter()
                .find(|method| method.name == "accept")
                .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
            let (target, _) = self.dispatch_target(class, method.slot, span)?;
            if matches!(target, Callee::Builtin(Builtin::ParentAccept)) {
                state.phase = FilterPhase::AcceptParentChild;
            }
        }
        let (receiver, name, extra) = match state.phase {
            FilterPhase::Rewind => (&state.inner, "rewind", Vec::new()),
            FilterPhase::CheckForAdvance => (&state.wrapper, "valid", Vec::new()),
            FilterPhase::CheckValid => (&state.inner, "valid", Vec::new()),
            FilterPhase::ReadValue => (&state.inner, "value", Vec::new()),
            FilterPhase::ReadKey => (&state.inner, "key", Vec::new()),
            FilterPhase::Accept => (
                &state.wrapper,
                "accept",
                vec![
                    state.value.as_ref().expect("filter value").clone(),
                    state.key.as_ref().expect("filter key").clone(),
                ],
            ),
            FilterPhase::AcceptParentChild | FilterPhase::ReadChild => (
                state.value.as_ref().expect("recursive filter entry"),
                "children",
                Vec::new(),
            ),
            FilterPhase::SkipAdvance | FilterPhase::Advance => {
                (&state.inner, "advance", Vec::new())
            }
            FilterPhase::FinalValue => (
                &state.inner,
                if state.builtin == Builtin::IteratorKey {
                    "key"
                } else {
                    "value"
                },
                Vec::new(),
            ),
            FilterPhase::ReadWrappedValue => (
                state.value.as_ref().expect("recursive filter entry"),
                "value",
                Vec::new(),
            ),
        };
        let class = receiver
            .class_id()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let method = self.program.classes[class.0 as usize]
            .methods
            .iter()
            .find(|method| method.name == name)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let ty = substitute_type_arguments(
            &method.return_type,
            method.declaring_class,
            receiver.type_arguments().unwrap_or_default(),
        );
        let (callee, _) = self.dispatch_target(class, method.slot, span)?;
        let mut arguments = vec![receiver.clone()];
        arguments.extend(extra);
        let mut instruction = state.instruction.clone();
        instruction.destination = Some(state.scratch);
        instruction.ty = Some(ty);
        frames[parent]
            .native_iterators
            .push(NativeIteratorContinuation::Filter(state));
        let native_depth = frames[parent].native_iterators.len();
        let result = self.task_call(frames, callee, arguments, Some(class), &instruction)?;
        let entry = &mut frames[parent];
        let nested_started = entry.native_iterators.len() > native_depth;
        let NativeIteratorContinuation::Filter(state) = entry
            .native_iterators
            .get_mut(native_depth - 1)
            .expect("filter caller")
        else {
            unreachable!("filter caller remains at its native depth");
        };
        match result {
            CallOutcome::Continue => {
                if !nested_started {
                    entry.frame.next_instruction -= 1;
                }
                state.pending = true;
                Ok(CallOutcome::Continue)
            }
            CallOutcome::Suspend(wait) => {
                if !matches!(wait, Wait::FileRetry { .. } | Wait::WorkerRetry { .. }) {
                    state.pending = true;
                    state.restore_instruction = true;
                }
                Ok(CallOutcome::Suspend(wait))
            }
        }
    }

    fn advance_limit(&mut self, frames: &mut Vec<TaskFrame>) -> Result<CallOutcome, VmError> {
        let parent = frames.len() - 1;
        let NativeIteratorContinuation::Limit(mut state) = frames[parent]
            .native_iterators
            .pop()
            .expect("limit continuation")
        else {
            unreachable!("limit continuation is topmost");
        };
        let span = state.instruction.span;
        let finish = if state.pending {
            let frame = &mut frames[parent].frame;
            if state.restore_instruction {
                frame.next_instruction -= 1;
                state.restore_instruction = false;
            }
            let value = frame.registers[state.scratch.0 as usize]
                .take()
                .expect("limit method result");
            state.pending = false;
            match state.phase {
                LimitPhase::Rewind | LimitPhase::SeekAdvance => {
                    if matches!(state.phase, LimitPhase::SeekAdvance) {
                        state.remaining -= 1;
                    }
                    if state.remaining > 0 {
                        state.phase = LimitPhase::SeekValid;
                        None
                    } else {
                        state
                            .wrapper
                            .set_property(PropertyId(3), Value::integer(0))
                            .map_err(|kind| runtime(kind, span))?;
                        Some(Value::NULL)
                    }
                }
                LimitPhase::SeekValid => {
                    if value.as_bool() == Some(true) {
                        self.tick(span)?;
                        state.phase = LimitPhase::SeekAdvance;
                        None
                    } else {
                        state
                            .wrapper
                            .set_property(PropertyId(3), Value::integer(0))
                            .map_err(|kind| runtime(kind, span))?;
                        Some(Value::NULL)
                    }
                }
                LimitPhase::Valid => Some(Value::bool(value.as_bool() == Some(true))),
                LimitPhase::Guard => {
                    if value.as_bool() == Some(true) {
                        state.phase = if state.builtin == Builtin::IteratorAdvance {
                            LimitPhase::Advance
                        } else {
                            LimitPhase::FinalValue
                        };
                        None
                    } else if state.builtin == Builtin::IteratorAdvance {
                        Some(Value::NULL)
                    } else {
                        return Err(runtime(
                            RuntimeErrorKind::Bounds("iterator is exhausted".to_owned()),
                            span,
                        ));
                    }
                }
                LimitPhase::FinalValue => Some(value),
                LimitPhase::Advance => {
                    state
                        .wrapper
                        .set_property(
                            PropertyId(3),
                            Value::integer(state.position.saturating_add(1)),
                        )
                        .map_err(|kind| runtime(kind, span))?;
                    Some(Value::NULL)
                }
            }
        } else {
            None
        };
        if let Some(value) = finish {
            let frame = &mut frames[parent].frame;
            frame.registers.pop();
            if let Some(destination) = state.instruction.destination {
                frame.registers[destination.0 as usize] = Some(value);
            }
            if !state.nested {
                frame.next_instruction += 1;
            }
            return Ok(CallOutcome::Continue);
        }
        let name = match state.phase {
            LimitPhase::Rewind => "rewind",
            LimitPhase::SeekValid | LimitPhase::Valid | LimitPhase::Guard => "valid",
            LimitPhase::SeekAdvance | LimitPhase::Advance => "advance",
            LimitPhase::FinalValue if state.builtin == Builtin::IteratorKey => "key",
            LimitPhase::FinalValue => "value",
        };
        let class = state
            .inner
            .class_id()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let method = self.program.classes[class.0 as usize]
            .methods
            .iter()
            .find(|method| method.name == name)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let ty = substitute_type_arguments(
            &method.return_type,
            method.declaring_class,
            state.inner.type_arguments().unwrap_or_default(),
        );
        let (callee, _) = self.dispatch_target(class, method.slot, span)?;
        let mut instruction = state.instruction.clone();
        instruction.destination = Some(state.scratch);
        instruction.ty = Some(ty);
        let inner = state.inner.clone();
        frames[parent]
            .native_iterators
            .push(NativeIteratorContinuation::Limit(state));
        let native_depth = frames[parent].native_iterators.len();
        let result = self.task_call(frames, callee, vec![inner], Some(class), &instruction)?;
        let entry = &mut frames[parent];
        let nested_started = entry.native_iterators.len() > native_depth;
        let NativeIteratorContinuation::Limit(state) = entry
            .native_iterators
            .get_mut(native_depth - 1)
            .expect("limit caller")
        else {
            unreachable!("limit caller remains at its native depth");
        };
        match result {
            CallOutcome::Continue => {
                if !nested_started {
                    entry.frame.next_instruction -= 1;
                }
                state.pending = true;
                Ok(CallOutcome::Continue)
            }
            CallOutcome::Suspend(wait) => {
                if !matches!(wait, Wait::FileRetry { .. } | Wait::WorkerRetry { .. }) {
                    state.pending = true;
                    state.restore_instruction = true;
                }
                Ok(CallOutcome::Suspend(wait))
            }
        }
    }

    fn advance_append(&mut self, frames: &mut Vec<TaskFrame>) -> Result<CallOutcome, VmError> {
        let parent = frames.len() - 1;
        let NativeIteratorContinuation::Append(mut state) = frames[parent]
            .native_iterators
            .pop()
            .expect("append continuation")
        else {
            unreachable!("append continuation is topmost");
        };
        let span = state.instruction.span;
        let finish = if state.pending {
            let frame = &mut frames[parent].frame;
            if state.restore_instruction {
                frame.next_instruction -= 1;
                state.restore_instruction = false;
            }
            let value = frame.registers[state.scratch.0 as usize]
                .take()
                .expect("append method result");
            state.pending = false;
            match state.phase {
                AppendPhase::Rewind => Some(Value::NULL),
                AppendPhase::CheckValid { after_advance } => {
                    if value.as_bool() == Some(true) {
                        state
                            .wrapper
                            .set_property(PropertyId(1), Value::integer(state.index))
                            .map_err(|kind| runtime(kind, span))?;
                        match state.builtin {
                            Builtin::IteratorValid => Some(Value::bool(true)),
                            Builtin::IteratorKey | Builtin::IteratorValue => {
                                state.phase = AppendPhase::FinalValue;
                                None
                            }
                            Builtin::IteratorAdvance if !after_advance => {
                                state.phase = AppendPhase::Advance;
                                None
                            }
                            Builtin::IteratorAdvance => Some(Value::NULL),
                            _ => unreachable!(),
                        }
                    } else {
                        self.tick(span)?;
                        state.index += 1;
                        if state.inner().is_some() {
                            state.phase = AppendPhase::RewindNext { after_advance };
                            None
                        } else {
                            state
                                .wrapper
                                .set_property(PropertyId(1), Value::integer(state.index))
                                .map_err(|kind| runtime(kind, span))?;
                            match state.builtin {
                                Builtin::IteratorValid => Some(Value::bool(false)),
                                Builtin::IteratorAdvance => Some(Value::NULL),
                                _ => {
                                    return Err(runtime(
                                        RuntimeErrorKind::Bounds(
                                            "iterator is exhausted".to_owned(),
                                        ),
                                        span,
                                    ));
                                }
                            }
                        }
                    }
                }
                AppendPhase::RewindNext { after_advance } => {
                    state.phase = AppendPhase::CheckValid { after_advance };
                    None
                }
                AppendPhase::FinalValue => Some(value),
                AppendPhase::Advance => {
                    state.phase = AppendPhase::CheckValid {
                        after_advance: true,
                    };
                    None
                }
            }
        } else {
            None
        };
        if let Some(value) = finish {
            let frame = &mut frames[parent].frame;
            frame.registers.pop();
            if let Some(destination) = state.instruction.destination {
                frame.registers[destination.0 as usize] = Some(value);
            }
            if !state.nested {
                frame.next_instruction += 1;
            }
            return Ok(CallOutcome::Continue);
        }
        let name = match state.phase {
            AppendPhase::Rewind | AppendPhase::RewindNext { .. } => "rewind",
            AppendPhase::CheckValid { .. } => "valid",
            AppendPhase::FinalValue if state.builtin == Builtin::IteratorKey => "key",
            AppendPhase::FinalValue => "value",
            AppendPhase::Advance => "advance",
        };
        let inner = state
            .inner()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let class = inner
            .class_id()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let method = self.program.classes[class.0 as usize]
            .methods
            .iter()
            .find(|method| method.name == name)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let ty = substitute_type_arguments(
            &method.return_type,
            method.declaring_class,
            inner.type_arguments().unwrap_or_default(),
        );
        let (callee, _) = self.dispatch_target(class, method.slot, span)?;
        let mut instruction = state.instruction.clone();
        instruction.destination = Some(state.scratch);
        instruction.ty = Some(ty);
        frames[parent]
            .native_iterators
            .push(NativeIteratorContinuation::Append(state));
        let native_depth = frames[parent].native_iterators.len();
        let result = self.task_call(frames, callee, vec![inner], Some(class), &instruction)?;
        let entry = &mut frames[parent];
        let nested_started = entry.native_iterators.len() > native_depth;
        let NativeIteratorContinuation::Append(state) = entry
            .native_iterators
            .get_mut(native_depth - 1)
            .expect("append caller")
        else {
            unreachable!("append caller remains at its native depth");
        };
        match result {
            CallOutcome::Continue => {
                if !nested_started {
                    entry.frame.next_instruction -= 1;
                }
                state.pending = true;
                Ok(CallOutcome::Continue)
            }
            CallOutcome::Suspend(wait) => {
                if !matches!(wait, Wait::FileRetry { .. } | Wait::WorkerRetry { .. }) {
                    state.pending = true;
                    state.restore_instruction = true;
                }
                Ok(CallOutcome::Suspend(wait))
            }
        }
    }

    fn advance_caching(&mut self, frames: &mut Vec<TaskFrame>) -> Result<CallOutcome, VmError> {
        let parent = frames.len() - 1;
        let NativeIteratorContinuation::Caching(mut state) = frames[parent]
            .native_iterators
            .pop()
            .expect("caching continuation")
        else {
            unreachable!("caching continuation is topmost");
        };
        let span = state.instruction.span;
        let mut finish = false;
        if state.pending {
            let frame = &mut frames[parent].frame;
            if state.restore_instruction {
                frame.next_instruction -= 1;
                state.restore_instruction = false;
            }
            let result = frame.registers[state.scratch.0 as usize]
                .take()
                .expect("caching method result");
            state.pending = false;
            match state.phase {
                CachingPhase::Rewind => {
                    let [key, value] = self
                        .native_type_arguments(&state.wrapper, "CachingIterator", span)?
                        .try_into()
                        .map_err(|_| runtime(RuntimeErrorKind::Unreachable, span))?;
                    let cache =
                        Value::try_map(key, value, vec![]).map_err(|kind| runtime(kind, span))?;
                    state
                        .wrapper
                        .set_property(PropertyId(8), cache)
                        .map_err(|kind| runtime(kind, span))?;
                    state
                        .wrapper
                        .set_property(PropertyId(9), Value::integer(0))
                        .map_err(|kind| runtime(kind, span))?;
                    state.phase = CachingPhase::PullValid;
                }
                CachingPhase::PullValid => {
                    if result.as_bool() == Some(true) {
                        state.phase = CachingPhase::PullKey;
                    } else {
                        state
                            .wrapper
                            .set_property(PropertyId(6 + state.pull), Value::bool(false))
                            .map_err(|kind| runtime(kind, span))?;
                        if state.pull == 0 {
                            state.pull = 1;
                            state.phase = CachingPhase::PullValid;
                        } else {
                            state.phase = CachingPhase::Store;
                        }
                    }
                }
                CachingPhase::PullKey => {
                    state.key = Some(result);
                    state.phase = CachingPhase::PullValue;
                }
                CachingPhase::PullValue => {
                    state.value = Some(result);
                    state.phase = CachingPhase::PullAdvance;
                }
                CachingPhase::PullAdvance => {
                    state
                        .wrapper
                        .set_property(
                            PropertyId(2 + 2 * state.pull),
                            state.key.take().expect("cached key"),
                        )
                        .map_err(|kind| runtime(kind, span))?;
                    state
                        .wrapper
                        .set_property(
                            PropertyId(3 + 2 * state.pull),
                            state.value.take().expect("cached value"),
                        )
                        .map_err(|kind| runtime(kind, span))?;
                    state
                        .wrapper
                        .set_property(PropertyId(6 + state.pull), Value::bool(true))
                        .map_err(|kind| runtime(kind, span))?;
                    if state.pull == 0 {
                        state.pull = 1;
                        state.phase = CachingPhase::PullValid;
                    } else {
                        state.phase = CachingPhase::Store;
                    }
                }
                CachingPhase::StoreStringCall => {
                    let flags = state
                        .wrapper
                        .property(PropertyId(1))
                        .map_err(|kind| runtime(kind, span))?
                        .as_int()
                        .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
                    Self::caching_commit_current(&state.wrapper, flags, span)?;
                    finish = true;
                }
                CachingPhase::Store => unreachable!("store is immediate"),
            }
        }
        let mut string_target = None;
        if matches!(state.phase, CachingPhase::Store) {
            let current = state
                .wrapper
                .property(PropertyId(6))
                .map_err(|kind| runtime(kind, span))?
                .as_bool()
                == Some(true);
            if current {
                let flags = state
                    .wrapper
                    .property(PropertyId(1))
                    .map_err(|kind| runtime(kind, span))?
                    .as_int()
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
                if flags & 1 != 0 {
                    let slot = if flags & 4 != 0 {
                        2
                    } else if flags & 16 != 0 {
                        0
                    } else {
                        3
                    };
                    let candidate = state
                        .wrapper
                        .property(PropertyId(slot))
                        .map_err(|kind| runtime(kind, span))?;
                    if candidate.output_bytes().is_none() {
                        let class = candidate.class_id().ok_or_else(|| {
                            self.native_exception(
                                "TypeError",
                                b"current value is not Stringable".to_vec(),
                                None,
                                0,
                                span,
                            )
                        })?;
                        if !is_instance_of_name(self.program, class, "Stringable") {
                            return Err(self.native_exception(
                                "TypeError",
                                b"current value is not Stringable".to_vec(),
                                None,
                                0,
                                span,
                            ));
                        }
                        string_target = Some(candidate);
                        state.phase = CachingPhase::StoreStringCall;
                    }
                }
                if string_target.is_none() {
                    Self::caching_commit_current(&state.wrapper, flags, span)?;
                    finish = true;
                }
            } else {
                finish = true;
            }
        }
        if finish {
            let frame = &mut frames[parent].frame;
            frame.registers.pop();
            if let Some(destination) = state.instruction.destination {
                frame.registers[destination.0 as usize] = Some(Value::NULL);
            }
            if !state.nested {
                frame.next_instruction += 1;
            }
            return Ok(CallOutcome::Continue);
        }
        let (receiver, name) = if let Some(receiver) = string_target {
            (receiver, "__toString")
        } else {
            let name = match state.phase {
                CachingPhase::Rewind => "rewind",
                CachingPhase::PullValid => "valid",
                CachingPhase::PullKey => "key",
                CachingPhase::PullValue => "value",
                CachingPhase::PullAdvance => "advance",
                CachingPhase::Store | CachingPhase::StoreStringCall => unreachable!(),
            };
            (state.inner.clone(), name)
        };
        let class = receiver
            .class_id()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let method = self.program.classes[class.0 as usize]
            .methods
            .iter()
            .find(|method| method.name == name)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let ty = substitute_type_arguments(
            &method.return_type,
            method.declaring_class,
            receiver.type_arguments().unwrap_or_default(),
        );
        let (callee, _) = self.dispatch_target(class, method.slot, span)?;
        let mut instruction = state.instruction.clone();
        instruction.destination = Some(state.scratch);
        instruction.ty = Some(ty);
        frames[parent]
            .native_iterators
            .push(NativeIteratorContinuation::Caching(state));
        let native_depth = frames[parent].native_iterators.len();
        let result = self.task_call(frames, callee, vec![receiver], Some(class), &instruction)?;
        let entry = &mut frames[parent];
        let nested_started = entry.native_iterators.len() > native_depth;
        let NativeIteratorContinuation::Caching(state) = entry
            .native_iterators
            .get_mut(native_depth - 1)
            .expect("caching caller")
        else {
            unreachable!("caching caller remains at its native depth");
        };
        match result {
            CallOutcome::Continue => {
                if !nested_started {
                    entry.frame.next_instruction -= 1;
                }
                state.pending = true;
                Ok(CallOutcome::Continue)
            }
            CallOutcome::Suspend(wait) => {
                if !matches!(wait, Wait::FileRetry { .. } | Wait::WorkerRetry { .. }) {
                    state.pending = true;
                    state.restore_instruction = true;
                }
                Ok(CallOutcome::Suspend(wait))
            }
        }
    }

    fn advance_infinite(&mut self, frames: &mut Vec<TaskFrame>) -> Result<CallOutcome, VmError> {
        let parent = frames.len() - 1;
        let NativeIteratorContinuation::Infinite(mut state) = frames[parent]
            .native_iterators
            .pop()
            .expect("infinite continuation")
        else {
            unreachable!("infinite continuation is topmost");
        };
        let span = state.instruction.span;
        let mut finish = None;
        if state.pending {
            let frame = &mut frames[parent].frame;
            if state.restore_instruction {
                frame.next_instruction -= 1;
                state.restore_instruction = false;
            }
            let result = frame.registers[state.scratch.0 as usize]
                .take()
                .expect("infinite method result");
            state.pending = false;
            match state.phase {
                InfinitePhase::FetchSource | InfinitePhase::FetchAggregate => {
                    state.fetched = Some(result);
                    state.phase = InfinitePhase::ResolveFetched;
                }
                InfinitePhase::RewindFetched => state.phase = InfinitePhase::ValidFetched,
                InfinitePhase::ValidFetched => {
                    if result.as_bool() == Some(true) {
                        let inner = state.fetched.take().expect("fetched iterator");
                        state
                            .wrapper
                            .set_property(PropertyId(1), inner.clone())
                            .map_err(|kind| runtime(kind, span))?;
                        state.inner = Some(inner);
                        match state.builtin {
                            Builtin::IteratorRewind | Builtin::IteratorAdvance => {
                                finish = Some(Value::NULL);
                            }
                            Builtin::IteratorValid => finish = Some(Value::bool(true)),
                            _ => state.phase = InfinitePhase::FinalMethod,
                        }
                    } else {
                        state
                            .wrapper
                            .set_property(PropertyId(2), Value::bool(true))
                            .map_err(|kind| runtime(kind, span))?;
                        state
                            .wrapper
                            .set_property(PropertyId(1), Value::NULL)
                            .map_err(|kind| runtime(kind, span))?;
                        finish = Some(match state.builtin {
                            Builtin::IteratorRewind | Builtin::IteratorAdvance => Value::NULL,
                            Builtin::IteratorValid => Value::bool(false),
                            _ => {
                                return Err(runtime(
                                    RuntimeErrorKind::Bounds("iterator is exhausted".to_owned()),
                                    span,
                                ));
                            }
                        });
                    }
                }
                InfinitePhase::CheckInnerValid { after_advance } => {
                    if result.as_bool() == Some(true) {
                        match state.builtin {
                            Builtin::IteratorAdvance if !after_advance => {
                                state.phase = InfinitePhase::AdvanceInner;
                            }
                            Builtin::IteratorAdvance => finish = Some(Value::NULL),
                            Builtin::IteratorValid => finish = Some(Value::bool(true)),
                            _ => state.phase = InfinitePhase::FinalMethod,
                        }
                    } else {
                        state.aggregate_depth = 0;
                        state.phase = InfinitePhase::FetchSource;
                    }
                }
                InfinitePhase::AdvanceInner => {
                    state.phase = InfinitePhase::CheckInnerValid {
                        after_advance: true,
                    };
                }
                InfinitePhase::FinalMethod => finish = Some(result),
                InfinitePhase::ResolveFetched => unreachable!("resolve is immediate"),
            }
        }
        if let Some(value) = finish {
            let frame = &mut frames[parent].frame;
            frame.registers.pop();
            if let Some(destination) = state.instruction.destination {
                frame.registers[destination.0 as usize] = Some(value);
            }
            if !state.nested {
                frame.next_instruction += 1;
            }
            return Ok(CallOutcome::Continue);
        }
        if matches!(state.phase, InfinitePhase::ResolveFetched) {
            let fetched = state.fetched.as_ref().expect("fetched traversable");
            let class = fetched
                .class_id()
                .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
            if is_instance_of_name(self.program, class, "Iterator") {
                state.phase = InfinitePhase::RewindFetched;
            } else {
                if state.aggregate_depth >= 64 {
                    return Err(self.native_exception(
                        "Error",
                        b"iterator aggregate nesting exceeds 64 levels".to_vec(),
                        None,
                        0,
                        span,
                    ));
                }
                self.tick(span)?;
                state.aggregate_depth += 1;
                state.phase = InfinitePhase::FetchAggregate;
            }
        }
        let (receiver, name) = match state.phase {
            InfinitePhase::FetchSource => (
                state
                    .wrapper
                    .property(PropertyId(0))
                    .map_err(|kind| runtime(kind, span))?,
                "getIterator",
            ),
            InfinitePhase::FetchAggregate => (
                state.fetched.as_ref().expect("fetched aggregate").clone(),
                "getIterator",
            ),
            InfinitePhase::RewindFetched => (
                state.fetched.as_ref().expect("fetched iterator").clone(),
                "rewind",
            ),
            InfinitePhase::ValidFetched => (
                state.fetched.as_ref().expect("fetched iterator").clone(),
                "valid",
            ),
            InfinitePhase::CheckInnerValid { .. } => (
                state.inner.as_ref().expect("current iterator").clone(),
                "valid",
            ),
            InfinitePhase::AdvanceInner => (
                state.inner.as_ref().expect("current iterator").clone(),
                "advance",
            ),
            InfinitePhase::FinalMethod => (
                state.inner.as_ref().expect("current iterator").clone(),
                if state.builtin == Builtin::IteratorKey {
                    "key"
                } else {
                    "value"
                },
            ),
            InfinitePhase::ResolveFetched => unreachable!(),
        };
        let class = receiver
            .class_id()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let method = self.program.classes[class.0 as usize]
            .methods
            .iter()
            .find(|method| method.name == name)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let ty = substitute_type_arguments(
            &method.return_type,
            method.declaring_class,
            receiver.type_arguments().unwrap_or_default(),
        );
        let (callee, _) = self.dispatch_target(class, method.slot, span)?;
        let mut instruction = state.instruction.clone();
        instruction.destination = Some(state.scratch);
        instruction.ty = Some(ty);
        frames[parent]
            .native_iterators
            .push(NativeIteratorContinuation::Infinite(state));
        let native_depth = frames[parent].native_iterators.len();
        let result = self.task_call(frames, callee, vec![receiver], Some(class), &instruction)?;
        let entry = &mut frames[parent];
        let nested_started = entry.native_iterators.len() > native_depth;
        let NativeIteratorContinuation::Infinite(state) = entry
            .native_iterators
            .get_mut(native_depth - 1)
            .expect("infinite caller")
        else {
            unreachable!("infinite caller remains at its native depth");
        };
        match result {
            CallOutcome::Continue => {
                if !nested_started {
                    entry.frame.next_instruction -= 1;
                }
                state.pending = true;
                Ok(CallOutcome::Continue)
            }
            CallOutcome::Suspend(wait) => {
                if !matches!(wait, Wait::FileRetry { .. } | Wait::WorkerRetry { .. }) {
                    state.pending = true;
                    state.restore_instruction = true;
                }
                Ok(CallOutcome::Suspend(wait))
            }
        }
    }

    fn advance_regex(&mut self, frames: &mut Vec<TaskFrame>) -> Result<CallOutcome, VmError> {
        let parent = frames.len() - 1;
        let NativeIteratorContinuation::Regex(mut state) = frames[parent]
            .native_iterators
            .pop()
            .expect("regex continuation")
        else {
            unreachable!("regex continuation is topmost");
        };
        let span = state.instruction.span;
        let mut finish = None;
        if state.pending {
            let frame = &mut frames[parent].frame;
            if state.restore_instruction {
                frame.next_instruction -= 1;
                state.restore_instruction = false;
            }
            let result = frame.registers[state.scratch.0 as usize]
                .take()
                .expect("regex method result");
            state.pending = false;
            match state.phase {
                RegexPhase::Rewind | RegexPhase::AdvanceInner => {
                    state
                        .wrapper
                        .set_property(PropertyId(6), Value::bool(false))
                        .map_err(|kind| runtime(kind, span))?;
                    finish = Some(Value::NULL);
                }
                RegexPhase::CheckForAdvance => {
                    if result.as_bool() == Some(true) {
                        state.phase = RegexPhase::AdvanceInner;
                    } else {
                        state
                            .wrapper
                            .set_property(PropertyId(6), Value::bool(false))
                            .map_err(|kind| runtime(kind, span))?;
                        finish = Some(Value::NULL);
                    }
                }
                RegexPhase::CheckValid => {
                    if result.as_bool() == Some(true) {
                        self.tick(span)?;
                        state.phase = RegexPhase::ReadKey;
                    } else if state.builtin == Builtin::IteratorValid {
                        finish = Some(Value::bool(false));
                    } else {
                        return Err(runtime(
                            RuntimeErrorKind::Bounds("iterator is exhausted".to_owned()),
                            span,
                        ));
                    }
                }
                RegexPhase::ReadKey => {
                    state.key = Some(result);
                    state.phase = RegexPhase::ReadEntry;
                }
                RegexPhase::ReadEntry => {
                    state.entry = Some(result.clone());
                    if state.options.as_ref().expect("scan options").recursive {
                        state.phase = RegexPhase::ReadNodeValue;
                    } else {
                        state.value = Some(result);
                        state.phase = RegexPhase::Evaluate;
                    }
                }
                RegexPhase::ReadNodeValue => {
                    state.value = Some(result);
                    state.phase = RegexPhase::Evaluate;
                }
                RegexPhase::ReadChildren => {
                    state.child = Some(result);
                    state.phase = RegexPhase::Decide;
                }
                RegexPhase::SkipAdvance => state.phase = RegexPhase::CheckValid,
                RegexPhase::FinalKey => finish = Some(result),
                RegexPhase::Evaluate | RegexPhase::Decide => {
                    unreachable!("regex step is immediate")
                }
            }
        }
        if matches!(state.phase, RegexPhase::Evaluate) {
            let options = state.options.as_ref().expect("scan options");
            let subject = if options.mode == 5 {
                state.key.as_ref().expect("regex key").as_bytes()
            } else {
                state.value.as_ref().expect("regex value").as_bytes()
            }
            .ok_or_else(|| {
                runtime(
                    RuntimeErrorKind::TypeError("regex input must be a string".to_owned()),
                    span,
                )
            })?;
            state.matched = Some(
                state
                    .regex
                    .as_ref()
                    .expect("compiled regex")
                    .is_match(subject)
                    .map_err(|error| {
                        runtime(
                            RuntimeErrorKind::TypeError(format!("PCRE2 match failed: {error}")),
                            span,
                        )
                    })?,
            );
            state.phase = if options.recursive {
                RegexPhase::ReadChildren
            } else {
                RegexPhase::Decide
            };
        }
        if matches!(state.phase, RegexPhase::Decide) {
            let options = state.options.as_ref().expect("scan options");
            let child = state.child.take().unwrap_or(Value::NULL);
            if state.matched.take().expect("regex match") != options.invert
                || (options.recursive && !child.is_null())
            {
                let value = state.value.as_ref().expect("regex value");
                let subject = if options.mode == 5 {
                    state.key.as_ref().expect("regex key").as_bytes()
                } else {
                    value.as_bytes()
                }
                .expect("scan validated string subject");
                self.regex_iterator_publish(
                    &state.wrapper,
                    (value, subject),
                    child,
                    state.regex.as_ref().expect("compiled regex"),
                    (
                        options.mode,
                        options.flags,
                        options
                            .replacement
                            .as_bytes()
                            .expect("verified replacement"),
                    ),
                    span,
                )?;
                match state.builtin {
                    Builtin::IteratorValid => finish = Some(Value::bool(true)),
                    Builtin::IteratorValue => {
                        finish = Some(
                            state
                                .wrapper
                                .property(PropertyId(7))
                                .map_err(|kind| runtime(kind, span))?,
                        );
                    }
                    Builtin::IteratorKey => state.phase = RegexPhase::FinalKey,
                    _ => unreachable!(),
                }
            } else {
                state.phase = RegexPhase::SkipAdvance;
            }
        }
        if let Some(value) = finish {
            let frame = &mut frames[parent].frame;
            frame.registers.pop();
            if let Some(destination) = state.instruction.destination {
                frame.registers[destination.0 as usize] = Some(value);
            }
            if !state.nested {
                frame.next_instruction += 1;
            }
            return Ok(CallOutcome::Continue);
        }
        let (receiver, name) = match state.phase {
            RegexPhase::Rewind => (state.inner.clone(), "rewind"),
            RegexPhase::CheckForAdvance => (state.wrapper.clone(), "valid"),
            RegexPhase::CheckValid => (state.inner.clone(), "valid"),
            RegexPhase::ReadKey | RegexPhase::FinalKey => (state.inner.clone(), "key"),
            RegexPhase::ReadEntry => (state.inner.clone(), "value"),
            RegexPhase::ReadNodeValue => (
                state.entry.as_ref().expect("recursive regex entry").clone(),
                "value",
            ),
            RegexPhase::ReadChildren => (
                state.entry.as_ref().expect("recursive regex entry").clone(),
                "children",
            ),
            RegexPhase::SkipAdvance | RegexPhase::AdvanceInner => (state.inner.clone(), "advance"),
            RegexPhase::Evaluate | RegexPhase::Decide => unreachable!(),
        };
        let class = receiver
            .class_id()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let method = self.program.classes[class.0 as usize]
            .methods
            .iter()
            .find(|method| method.name == name)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let ty = substitute_type_arguments(
            &method.return_type,
            method.declaring_class,
            receiver.type_arguments().unwrap_or_default(),
        );
        let (callee, _) = self.dispatch_target(class, method.slot, span)?;
        let mut instruction = state.instruction.clone();
        instruction.destination = Some(state.scratch);
        instruction.ty = Some(ty);
        frames[parent]
            .native_iterators
            .push(NativeIteratorContinuation::Regex(state));
        let native_depth = frames[parent].native_iterators.len();
        let result = self.task_call(frames, callee, vec![receiver], Some(class), &instruction)?;
        let entry = &mut frames[parent];
        let nested_started = entry.native_iterators.len() > native_depth;
        let NativeIteratorContinuation::Regex(state) = entry
            .native_iterators
            .get_mut(native_depth - 1)
            .expect("regex caller")
        else {
            unreachable!("regex caller remains at its native depth");
        };
        match result {
            CallOutcome::Continue => {
                if !nested_started {
                    entry.frame.next_instruction -= 1;
                }
                state.pending = true;
                Ok(CallOutcome::Continue)
            }
            CallOutcome::Suspend(wait) => {
                if !matches!(wait, Wait::FileRetry { .. } | Wait::WorkerRetry { .. }) {
                    state.pending = true;
                    state.restore_instruction = true;
                }
                Ok(CallOutcome::Suspend(wait))
            }
        }
    }

    fn advance_walk(&mut self, frames: &mut Vec<TaskFrame>) -> Result<CallOutcome, VmError> {
        let parent = frames.len() - 1;
        let NativeIteratorContinuation::Walk(mut state) = frames[parent]
            .native_iterators
            .pop()
            .expect("walk continuation")
        else {
            unreachable!("walk continuation is topmost");
        };
        let span = state.instruction.span;
        let mut finish = false;
        if state.pending {
            let frame = &mut frames[parent].frame;
            if state.restore_instruction {
                frame.next_instruction -= 1;
                state.restore_instruction = false;
            }
            let result = frame.registers[state.scratch.0 as usize]
                .take()
                .expect("walk method result");
            state.pending = false;
            match state.phase {
                WalkPhase::BeginIteration => state.phase = WalkPhase::RewindRoot,
                WalkPhase::RewindRoot => {
                    let root = state
                        .wrapper
                        .property(PropertyId(0))
                        .map_err(|kind| runtime(kind, span))?;
                    state.frames.push(WalkFrame::new(root));
                    Self::store_walk_stack(&state.wrapper, state.frames.clone(), span)?;
                    state.phase = WalkPhase::Seek;
                }
                WalkPhase::CheckValid => {
                    if result.as_bool() == Some(true) {
                        state.phase = WalkPhase::ReadEntry;
                    } else {
                        state.frames.pop();
                        if state.frames.is_empty() {
                            state.phase = WalkPhase::Seek;
                        } else {
                            Self::store_walk_stack(&state.wrapper, state.frames.clone(), span)?;
                            state.phase = WalkPhase::EndChildren;
                        }
                    }
                }
                WalkPhase::ReadEntry => {
                    state.frames.last_mut().expect("walk frame").entry = result;
                    state.phase = WalkPhase::ReadChild;
                }
                WalkPhase::ReadChild => {
                    let top = state.frames.len() - 1;
                    state.frames[top].child = result.clone();
                    if state.mode == 1 {
                        state.frames[top].phase = 1;
                        state.phase = WalkPhase::PublishKey;
                    } else if !result.is_null()
                        && (state.max_depth < 0
                            || i64::try_from(top).is_ok_and(|level| level < state.max_depth))
                    {
                        state.frames[top].phase = if state.mode == 2 { 2 } else { 3 };
                        Self::store_walk_stack(&state.wrapper, state.frames.clone(), span)?;
                        state.phase = WalkPhase::BeginChildren;
                    } else {
                        state.frames[top].phase = 4;
                        state.phase = WalkPhase::PublishKey;
                    }
                }
                WalkPhase::BeginChildren => state.phase = WalkPhase::RewindChild,
                WalkPhase::RewindChild => {
                    let child = state.frames.last().expect("walk frame").child.clone();
                    state.frames.push(WalkFrame::new(child));
                    state.phase = WalkPhase::Seek;
                }
                WalkPhase::EndChildren => state.phase = WalkPhase::Seek,
                WalkPhase::PublishKey => {
                    state.key = Some(result);
                    state.phase = WalkPhase::PublishValue;
                }
                WalkPhase::PublishValue => {
                    state
                        .wrapper
                        .set_property(PropertyId(4), state.key.take().expect("walk key"))
                        .map_err(|kind| runtime(kind, span))?;
                    state
                        .wrapper
                        .set_property(PropertyId(5), result)
                        .map_err(|kind| runtime(kind, span))?;
                    state
                        .wrapper
                        .set_property(
                            PropertyId(6),
                            Value::integer(
                                i64::try_from(state.frames.len() - 1)
                                    .map_err(|_| runtime(RuntimeErrorKind::Unreachable, span))?,
                            ),
                        )
                        .map_err(|kind| runtime(kind, span))?;
                    state
                        .wrapper
                        .set_property(PropertyId(7), Value::bool(true))
                        .map_err(|kind| runtime(kind, span))?;
                    Self::store_walk_stack(&state.wrapper, state.frames.clone(), span)?;
                    state.phase = WalkPhase::NextElement;
                }
                WalkPhase::NextElement | WalkPhase::EndIteration => finish = true,
                WalkPhase::AdvanceCursor => {
                    state.frames.last_mut().expect("walk frame").phase = 0;
                    state.phase = WalkPhase::Seek;
                }
                WalkPhase::Seek => unreachable!("seek is immediate"),
            }
        }
        while matches!(state.phase, WalkPhase::Seek) {
            self.tick(span)?;
            let Some(top) = state.frames.len().checked_sub(1) else {
                state
                    .wrapper
                    .set_property(PropertyId(7), Value::bool(false))
                    .map_err(|kind| runtime(kind, span))?;
                Self::store_walk_stack(&state.wrapper, state.frames.clone(), span)?;
                let ended = state
                    .wrapper
                    .property(PropertyId(9))
                    .map_err(|kind| runtime(kind, span))?
                    .as_bool()
                    == Some(true);
                if ended {
                    finish = true;
                } else {
                    state
                        .wrapper
                        .set_property(PropertyId(9), Value::bool(true))
                        .map_err(|kind| runtime(kind, span))?;
                    state.phase = WalkPhase::EndIteration;
                }
                break;
            };
            match state.frames[top].phase {
                0 => state.phase = WalkPhase::CheckValid,
                1 => {
                    let child = &state.frames[top].child;
                    let descend = !child.is_null()
                        && (state.max_depth < 0
                            || i64::try_from(top).is_ok_and(|level| level < state.max_depth));
                    state.frames[top].phase = if descend { 3 } else { 4 };
                    if descend {
                        Self::store_walk_stack(&state.wrapper, state.frames.clone(), span)?;
                        state.phase = WalkPhase::BeginChildren;
                    }
                }
                2 => {
                    state.frames[top].phase = 4;
                    state.phase = WalkPhase::PublishKey;
                }
                3 => state.frames[top].phase = 4,
                _ => state.phase = WalkPhase::AdvanceCursor,
            }
        }
        if finish {
            let frame = &mut frames[parent].frame;
            frame.registers.pop();
            if let Some(destination) = state.instruction.destination {
                frame.registers[destination.0 as usize] = Some(Value::NULL);
            }
            if !state.nested {
                frame.next_instruction += 1;
            }
            return Ok(CallOutcome::Continue);
        }
        let (receiver, name) = match state.phase {
            WalkPhase::BeginIteration => (state.wrapper.clone(), "beginIteration"),
            WalkPhase::RewindRoot => (
                state
                    .wrapper
                    .property(PropertyId(0))
                    .map_err(|kind| runtime(kind, span))?,
                "rewind",
            ),
            WalkPhase::CheckValid => (
                state.frames.last().expect("walk frame").iterator.clone(),
                "valid",
            ),
            WalkPhase::ReadEntry => (
                state.frames.last().expect("walk frame").iterator.clone(),
                "value",
            ),
            WalkPhase::ReadChild => (
                state.frames.last().expect("walk frame").entry.clone(),
                "children",
            ),
            WalkPhase::BeginChildren => (state.wrapper.clone(), "beginChildren"),
            WalkPhase::RewindChild => (
                state.frames.last().expect("walk frame").child.clone(),
                "rewind",
            ),
            WalkPhase::EndChildren => (state.wrapper.clone(), "endChildren"),
            WalkPhase::PublishKey => (
                state.frames.last().expect("walk frame").iterator.clone(),
                "key",
            ),
            WalkPhase::PublishValue => (
                state.frames.last().expect("walk frame").entry.clone(),
                "value",
            ),
            WalkPhase::NextElement => (state.wrapper.clone(), "nextElement"),
            WalkPhase::EndIteration => (state.wrapper.clone(), "endIteration"),
            WalkPhase::AdvanceCursor => (
                state.frames.last().expect("walk frame").iterator.clone(),
                "advance",
            ),
            WalkPhase::Seek => unreachable!(),
        };
        let class = receiver
            .class_id()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let method = self.program.classes[class.0 as usize]
            .methods
            .iter()
            .find(|method| method.name == name)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let ty = substitute_type_arguments(
            &method.return_type,
            method.declaring_class,
            receiver.type_arguments().unwrap_or_default(),
        );
        let (callee, _) = self.dispatch_target(class, method.slot, span)?;
        let mut instruction = state.instruction.clone();
        instruction.destination = Some(state.scratch);
        instruction.ty = Some(ty);
        frames[parent]
            .native_iterators
            .push(NativeIteratorContinuation::Walk(state));
        let native_depth = frames[parent].native_iterators.len();
        let result = self.task_call(frames, callee, vec![receiver], Some(class), &instruction)?;
        let entry = &mut frames[parent];
        let nested_started = entry.native_iterators.len() > native_depth;
        let NativeIteratorContinuation::Walk(state) = entry
            .native_iterators
            .get_mut(native_depth - 1)
            .expect("walk caller")
        else {
            unreachable!("walk caller remains at its native depth");
        };
        match result {
            CallOutcome::Continue => {
                if !nested_started {
                    entry.frame.next_instruction -= 1;
                }
                state.pending = true;
                Ok(CallOutcome::Continue)
            }
            CallOutcome::Suspend(wait) => {
                if !matches!(wait, Wait::FileRetry { .. } | Wait::WorkerRetry { .. }) {
                    state.pending = true;
                    state.restore_instruction = true;
                }
                Ok(CallOutcome::Suspend(wait))
            }
        }
    }

    fn advance_limit_seek(&mut self, frames: &mut Vec<TaskFrame>) -> Result<CallOutcome, VmError> {
        let parent = frames.len() - 1;
        let NativeIteratorContinuation::LimitSeek(mut state) = frames[parent]
            .native_iterators
            .pop()
            .expect("limit seek continuation")
        else {
            unreachable!("limit seek continuation is topmost");
        };
        let span = state.instruction.span;
        let mut finish = false;
        if state.pending {
            let frame = &mut frames[parent].frame;
            if state.restore_instruction {
                frame.next_instruction -= 1;
                state.restore_instruction = false;
            }
            let result = frame.registers[state.scratch.0 as usize]
                .take()
                .expect("limit seek method result");
            state.pending = false;
            match state.phase {
                LimitSeekPhase::Rewind => {
                    state.phase = if state.position == 0 {
                        LimitSeekPhase::FinalValid
                    } else {
                        LimitSeekPhase::CheckValid
                    };
                }
                LimitSeekPhase::CheckValid => {
                    if result.as_bool() == Some(true) {
                        self.tick(span)?;
                        state.phase = LimitSeekPhase::Advance;
                    } else {
                        state.phase = LimitSeekPhase::FinalValid;
                    }
                }
                LimitSeekPhase::Advance => {
                    state.index += 1;
                    state.phase = if state.index < state.position {
                        LimitSeekPhase::CheckValid
                    } else {
                        LimitSeekPhase::FinalValid
                    };
                }
                LimitSeekPhase::FinalValid => {
                    if result.as_bool() != Some(true) {
                        return Err(self.native_exception(
                            "OutOfBoundsException",
                            b"limit iterator position is out of bounds".to_vec(),
                            None,
                            0,
                            span,
                        ));
                    }
                    finish = true;
                }
            }
        }
        if finish {
            let frame = &mut frames[parent].frame;
            frame.registers.pop();
            if let Some(destination) = state.instruction.destination {
                frame.registers[destination.0 as usize] = Some(Value::NULL);
            }
            if !state.nested {
                frame.next_instruction += 1;
            }
            return Ok(CallOutcome::Continue);
        }
        let name = match state.phase {
            LimitSeekPhase::Rewind => "rewind",
            LimitSeekPhase::CheckValid | LimitSeekPhase::FinalValid => "valid",
            LimitSeekPhase::Advance => "advance",
        };
        let class = state
            .wrapper
            .class_id()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let method = self.program.classes[class.0 as usize]
            .methods
            .iter()
            .find(|method| method.name == name)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let ty = substitute_type_arguments(
            &method.return_type,
            method.declaring_class,
            state.wrapper.type_arguments().unwrap_or_default(),
        );
        let (callee, _) = self.dispatch_target(class, method.slot, span)?;
        let mut instruction = state.instruction.clone();
        instruction.destination = Some(state.scratch);
        instruction.ty = Some(ty);
        let wrapper = state.wrapper.clone();
        frames[parent]
            .native_iterators
            .push(NativeIteratorContinuation::LimitSeek(state));
        let native_depth = frames[parent].native_iterators.len();
        let result = self.task_call(frames, callee, vec![wrapper], Some(class), &instruction)?;
        let entry = &mut frames[parent];
        let nested_started = entry.native_iterators.len() > native_depth;
        let NativeIteratorContinuation::LimitSeek(state) = entry
            .native_iterators
            .get_mut(native_depth - 1)
            .expect("limit seek caller")
        else {
            unreachable!("limit seek caller remains at its native depth");
        };
        match result {
            CallOutcome::Continue => {
                if !nested_started {
                    entry.frame.next_instruction -= 1;
                }
                state.pending = true;
                Ok(CallOutcome::Continue)
            }
            CallOutcome::Suspend(wait) => {
                if !matches!(wait, Wait::FileRetry { .. } | Wait::WorkerRetry { .. }) {
                    state.pending = true;
                    state.restore_instruction = true;
                }
                Ok(CallOutcome::Suspend(wait))
            }
        }
    }

    fn file_open_request(
        &self,
        builtin: Builtin,
        arguments: &[Value],
        instruction: &Instruction,
    ) -> Result<Option<(PathBuf, OpenCompletion)>, VmError> {
        let span = instruction.span;
        let (path, flags, class, object) = match builtin {
            Builtin::FilesOpenRead | Builtin::FilesOpenWrite | Builtin::FilesOpenReadWrite => {
                let flags = match builtin {
                    Builtin::FilesOpenRead => (true, false, false, false, false, false),
                    Builtin::FilesOpenWrite | Builtin::FilesOpenReadWrite => {
                        let read = builtin == Builtin::FilesOpenReadWrite;
                        match arguments[1].as_int().expect("verified write mode") {
                            0 => (read, true, false, false, false, false),
                            1 => (read, true, false, true, true, false),
                            2 => (read, true, true, false, true, false),
                            3 => (read, true, false, false, true, false),
                            4 => (read, true, false, false, false, true),
                            _ => return Ok(None),
                        }
                    }
                    _ => unreachable!(),
                };
                (
                    arguments[0].as_bytes().expect("verified path").to_vec(),
                    flags,
                    self.result_class(instruction)?,
                    None,
                )
            }
            Builtin::StreamsOpen => {
                let uri = arguments[0].as_bytes().expect("verified URI string");
                let mode = arguments[1].as_int().expect("verified mode");
                if !(0..=6).contains(&mode)
                    || uri == b"thp:/input"
                    || uri == b"php://memory"
                    || uri == b"php://temp"
                    || uri.starts_with(b"php://temp/maxmemory:")
                {
                    return Ok(None);
                }
                let path = if uri.starts_with(b"file://") {
                    local_file_uri_path(uri).ok_or_else(|| {
                        self.native_exception(
                            "InvalidStreamUriException",
                            b"invalid file URI".to_vec(),
                            Some(uri.to_vec()),
                            0,
                            span,
                        )
                    })?
                } else if uri.windows(3).any(|part| part == b"://")
                    || uri.starts_with(b"php:")
                    || uri.starts_with(b"thp:")
                {
                    return Ok(None);
                } else {
                    uri.to_vec()
                };
                let flags = match mode {
                    0 => (true, false, false, false, false, false),
                    1 => (false, true, false, true, true, false),
                    2 => (true, true, false, false, false, false),
                    3 => (false, true, true, false, true, false),
                    4 => (true, true, false, true, true, false),
                    5 => (true, true, true, false, true, false),
                    6 => (true, true, false, false, false, true),
                    _ => unreachable!(),
                };
                let class = self.class_named(
                    match mode {
                        0 => "ReadableFileStream",
                        1 | 3 => "WritableFileStream",
                        _ => "ReadWriteFileStream",
                    },
                    span,
                )?;
                (path, flags, class, None)
            }
            Builtin::FileObject(FileObjectBuiltin::Construct) => (
                arguments[1]
                    .as_bytes()
                    .expect("verified file path")
                    .to_vec(),
                (true, false, false, false, false, false),
                self.class_named("ReadableFileStream", span)?,
                Some(arguments[0].clone()),
            ),
            _ => return Ok(None),
        };
        let resolved = self.resolve_local_stream_path(&path, span)?;
        let (read, write, append, truncate, create, exclusive) = flags;
        Ok(Some((
            resolved,
            OpenCompletion {
                path,
                class,
                flags: OpenFileFlags {
                    read,
                    write,
                    append,
                    truncate,
                    create,
                    exclusive,
                },
                object,
            },
        )))
    }

    fn task_generator_resume(
        &mut self,
        frames: &mut Vec<TaskFrame>,
        builtin: Builtin,
        iterator: &Value,
        instruction: &Instruction,
    ) -> Result<Option<CallOutcome>, VmError> {
        let Some(class) = iterator.class_id() else {
            return Ok(None);
        };
        if !is_instance_of_name(self.program, class, "Generator") {
            return Ok(None);
        }
        let span = instruction.span;
        let mut state = iterator.take_generator_state().ok_or_else(|| {
            self.native_exception(
                "Error",
                b"generator is already running".to_vec(),
                None,
                0,
                span,
            )
        })?;
        let mut close_marker = None;
        if builtin == Builtin::GeneratorClose {
            if matches!(
                state.status,
                GeneratorStatus::Exhausted | GeneratorStatus::Closed
            ) {
                iterator.put_generator_state(state);
                let frame = &mut frames.last_mut().expect("generator caller").frame;
                if let Some(destination) = instruction.destination {
                    frame.registers[destination.0 as usize] = Some(Value::NULL);
                }
                frame.next_instruction += 1;
                return Ok(Some(CallOutcome::Continue));
            }
            if state.status != GeneratorStatus::Suspended {
                state.status = GeneratorStatus::Closed;
                state.frame = GeneratorFrame::default();
                state.key = None;
                state.value = None;
                iterator.put_generator_state(state);
                let frame = &mut frames.last_mut().expect("generator caller").frame;
                if let Some(destination) = instruction.destination {
                    frame.registers[destination.0 as usize] = Some(Value::NULL);
                }
                frame.next_instruction += 1;
                return Ok(Some(CallOutcome::Continue));
            }
            let marker =
                match self.native_exception("Error", Vec::new(), None, GENERATOR_EXIT_CODE, span) {
                    VmError::Thrown { value, .. } => value,
                    error => {
                        iterator.put_generator_state(state);
                        return Err(error);
                    }
                };
            let mut frame = Frame::from_generator(std::mem::take(&mut state.frame));
            let function = &self.program.functions[state.function.0 as usize];
            let current = frame.current;
            if !catch_exception(self.program, function, &mut frame, current, marker.clone()) {
                state.status = GeneratorStatus::Closed;
                state.key = None;
                state.value = None;
                state.return_value = None;
                iterator.put_generator_state(state);
                let caller = &mut frames.last_mut().expect("generator caller").frame;
                if let Some(destination) = instruction.destination {
                    caller.registers[destination.0 as usize] = Some(Value::NULL);
                }
                caller.next_instruction += 1;
                return Ok(Some(CallOutcome::Continue));
            }
            state.frame = frame.into_generator();
            close_marker = Some(marker);
        }
        let resume = match builtin {
            Builtin::IteratorRewind => match state.status {
                GeneratorStatus::Fresh => true,
                GeneratorStatus::Suspended if !state.advanced => false,
                _ => {
                    iterator.put_generator_state(state);
                    return Err(self.native_exception(
                        "Error",
                        b"generator cannot be rewound".to_vec(),
                        None,
                        0,
                        span,
                    ));
                }
            },
            Builtin::IteratorAdvance => {
                if state.status == GeneratorStatus::Suspended {
                    state.advanced = true;
                    true
                } else {
                    false
                }
            }
            Builtin::GeneratorClose => true,
            _ => unreachable!(),
        };
        if !resume {
            iterator.put_generator_state(state);
            let frame = &mut frames.last_mut().expect("generator caller").frame;
            if let Some(destination) = instruction.destination {
                frame.registers[destination.0 as usize] = Some(Value::NULL);
            }
            frame.next_instruction += 1;
            return Ok(Some(CallOutcome::Continue));
        }
        let depth = frames.len();
        if let Some(limit) = self.limits.max_stack_depth
            && depth > limit
        {
            iterator.put_generator_state(state);
            return Err(VmError::StackDepthLimit { limit, span });
        }
        frames
            .last_mut()
            .expect("generator caller")
            .frame
            .next_instruction += 1;
        frames.push(TaskFrame {
            function: state.function,
            frame: Frame::from_generator(std::mem::take(&mut state.frame)),
            destination: instruction.destination,
            caller_span: span,
            validate_countable: false,
            return_override: None,
            reflection_return: None,
            generator: Some(GeneratorActivation {
                iterator: iterator.clone(),
                state,
                close_marker,
            }),
            collection: None,
            iterator: None,
            native_iterators: Vec::new(),
        });
        Ok(Some(CallOutcome::Continue))
    }

    fn directory_request(
        &self,
        operation: DirectoryBuiltin,
        arguments: &[Value],
        instruction: &Instruction,
    ) -> Option<(PathBuf, DirectoryCompletion)> {
        if !matches!(
            operation,
            DirectoryBuiltin::ConstructDirectory
                | DirectoryBuiltin::ConstructFilesystem
                | DirectoryBuiltin::ConstructRecursive
                | DirectoryBuiltin::ConstructGlob
        ) {
            return None;
        }
        let input =
            std::str::from_utf8(arguments[1].as_bytes().expect("verified directory path")).ok()?;
        if input.is_empty() || input.contains('\0') {
            return None;
        }
        let directory = if operation == DirectoryBuiltin::ConstructGlob {
            let path = Path::new(input);
            let parent = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let name = path.file_name()?.to_str()?;
            let parent = parent.to_str()?;
            if parent.contains(['*', '?', '[', ']']) || name.contains(['[', ']']) {
                return None;
            }
            parent
        } else {
            input
        };
        let path = if Path::new(directory).is_relative() {
            self.filesystem_base.join(directory)
        } else {
            PathBuf::from(directory)
        };
        Some((
            path,
            DirectoryCompletion {
                operation,
                arguments: arguments.to_vec(),
                instruction: instruction.clone(),
                directory: directory.to_owned(),
            },
        ))
    }

    fn recursive_directory_request(
        &self,
        object: &Value,
        instruction: &Instruction,
    ) -> Result<Option<(PathBuf, bool, RecursiveDirectoryCompletion)>, VmError> {
        let span = instruction.span;
        let entries = object
            .property(PropertyId(0))
            .map_err(|kind| runtime(kind, span))?;
        let entries = entries
            .vector_values()
            .expect("initialized directory entries");
        let index = object
            .property(PropertyId(1))
            .map_err(|kind| runtime(kind, span))?
            .as_int()
            .expect("initialized directory index");
        let Some(info) = usize::try_from(index)
            .ok()
            .and_then(|index| entries.get(index))
        else {
            return Ok(None);
        };
        let path_value = info
            .property(PropertyId(0))
            .map_err(|kind| runtime(kind, span))?;
        let path_bytes = path_value.as_bytes().expect("file info path");
        let path_text = std::str::from_utf8(path_bytes).expect("UTF-8 directory path");
        if matches!(directory_entry_name(path_text), "." | "..") {
            return Ok(None);
        }
        let path = if Path::new(path_text).is_relative() {
            self.filesystem_base.join(path_text)
        } else {
            PathBuf::from(path_text)
        };
        let follow = object
            .property(PropertyId(3))
            .map_err(|kind| runtime(kind, span))?
            .as_bool()
            == Some(true);
        Ok(Some((
            path,
            follow,
            RecursiveDirectoryCompletion {
                object: object.clone(),
                info: info.clone(),
                path: path_bytes.to_vec(),
                instruction: instruction.clone(),
            },
        )))
    }

    fn task_new_dynamic(
        &mut self,
        frames: &mut Vec<TaskFrame>,
        instruction: &Instruction,
    ) -> Result<(), VmError> {
        let InstructionKind::NewDynamic {
            target,
            type_arguments,
            arguments,
        } = &instruction.kind
        else {
            unreachable!("dynamic construction instruction")
        };
        let entry = frames.last().expect("task frame");
        let function = &self.program.functions[entry.function.0 as usize];
        let target = get_register(&entry.frame, *target, instruction.span)?;
        let values = arguments
            .iter()
            .map(|argument| {
                Ok((
                    argument.name.clone(),
                    get_register(&entry.frame, argument.value, argument.span)?,
                    function.register_types[argument.value.0 as usize].clone(),
                    argument.span,
                ))
            })
            .collect::<Result<Vec<_>, VmError>>()?;
        let prepared =
            self.prepare_dynamic_new(&target, type_arguments, values, function, instruction)?;
        match prepared.constructor {
            Some((Callee::Function(constructor), arguments, class))
                if !self.program.functions[constructor.0 as usize].generator =>
            {
                let depth = frames.len();
                if self
                    .limits
                    .max_stack_depth
                    .is_some_and(|limit| depth > limit)
                {
                    return Err(VmError::StackDepthLimit {
                        limit: self.limits.max_stack_depth.expect("limit exists"),
                        span: instruction.span,
                    });
                }
                frames
                    .last_mut()
                    .expect("task frame")
                    .frame
                    .next_instruction += 1;
                frames.push(TaskFrame {
                    function: constructor,
                    frame: self.new_frame(constructor, arguments, Some(class)),
                    destination: instruction.destination,
                    caller_span: instruction.span,
                    validate_countable: false,
                    return_override: Some(prepared.object),
                    reflection_return: None,
                    generator: None,
                    collection: None,
                    iterator: None,
                    native_iterators: Vec::new(),
                });
                return Ok(());
            }
            Some((callee, arguments, class)) => {
                self.invoke_callee(
                    callee,
                    arguments,
                    frames.len(),
                    Some(class),
                    function,
                    instruction,
                )?;
            }
            None => {}
        }
        let entry = &mut frames.last_mut().expect("task frame").frame;
        if let Some(destination) = instruction.destination {
            entry.registers[destination.0 as usize] = Some(prepared.object);
        }
        entry.next_instruction += 1;
        Ok(())
    }

    fn file_operation(
        &self,
        builtin: Builtin,
        arguments: &[Value],
        span: Span,
    ) -> Result<Option<StreamFileOperation>, VmError> {
        Ok(match builtin {
            Builtin::StreamRead => Some(StreamFileOperation::Read(self.nonnegative_stream_value(
                arguments[1].as_int().expect("verified read length"),
                "stream read length",
                span,
            )?)),
            Builtin::StreamReadAll => Some(StreamFileOperation::ReadAll(
                arguments
                    .get(1)
                    .filter(|value| !value.is_null())
                    .map(|value| {
                        self.nonnegative_stream_value(
                            value.as_int().expect("verified read limit"),
                            "stream read limit",
                            span,
                        )
                    })
                    .transpose()?,
            )),
            Builtin::StreamWrite | Builtin::StreamWriteAll => Some(StreamFileOperation::Write(
                arguments[1]
                    .as_bytes()
                    .expect("verified write bytes")
                    .to_vec(),
            )),
            Builtin::StreamFlush => Some(StreamFileOperation::Flush),
            Builtin::StreamEof => Some(StreamFileOperation::Eof),
            Builtin::StreamSeek => Some(StreamFileOperation::Seek {
                offset: arguments[1].as_int().expect("verified seek offset"),
                origin: arguments[2].as_int().expect("verified seek origin"),
            }),
            _ => None,
        })
    }

    fn file_result_value(
        &self,
        completion: &FileCompletion,
        result: Result<StreamFileResult, RuntimeErrorKind>,
        span: Span,
    ) -> Result<Value, VmError> {
        let result = result.map_err(|kind| {
            if matches!(kind, RuntimeErrorKind::Bounds(_)) {
                self.native_exception("ValueError", kind.to_string().into_bytes(), None, 0, span)
            } else {
                self.stream_exception(kind, span)
            }
        })?;
        match (completion, result) {
            (
                FileCompletion::Stream(Builtin::StreamRead | Builtin::StreamReadAll),
                StreamFileResult::Bytes(bytes),
            ) => Value::try_bytes(bytes).map_err(|kind| runtime(kind, span)),
            (FileCompletion::Stream(Builtin::StreamWrite), StreamFileResult::Written(count)) => {
                Ok(Value::integer(i64::try_from(count).map_err(|_| {
                    runtime(
                        RuntimeErrorKind::Io("write count overflow".to_owned()),
                        span,
                    )
                })?))
            }
            (FileCompletion::Stream(Builtin::StreamWriteAll), StreamFileResult::Written(_))
            | (FileCompletion::Stream(Builtin::StreamFlush), StreamFileResult::Flushed) => {
                Ok(Value::NULL)
            }
            (
                FileCompletion::Stream(Builtin::StreamEof)
                | FileCompletion::FileObject {
                    operation: FileObjectBuiltin::Eof,
                    ..
                },
                StreamFileResult::Bool(value),
            ) => Ok(Value::bool(value)),
            (FileCompletion::Stream(Builtin::StreamSeek), StreamFileResult::Position(position)) => {
                Ok(Value::integer(i64::try_from(position).map_err(|_| {
                    runtime(
                        RuntimeErrorKind::Io("stream position overflow".to_owned()),
                        span,
                    )
                })?))
            }
            (
                FileCompletion::FileObject {
                    operation: FileObjectBuiltin::Fread,
                    ..
                },
                StreamFileResult::Bytes(bytes),
            ) => Value::try_bytes(bytes).map_err(|kind| runtime(kind, span)),
            (
                FileCompletion::FileObject {
                    operation: FileObjectBuiltin::Fgets,
                    ..
                },
                StreamFileResult::Line(bytes),
            ) => Value::try_bytes(bytes.unwrap_or_default()).map_err(|kind| runtime(kind, span)),
            (
                FileCompletion::FileObject {
                    operation,
                    object,
                    line,
                },
                StreamFileResult::Line(bytes),
            ) if matches!(
                operation,
                FileObjectBuiltin::Rewind | FileObjectBuiltin::Advance | FileObjectBuiltin::Seek
            ) =>
            {
                if *operation == FileObjectBuiltin::Seek && bytes.is_none() {
                    return Err(self.native_exception(
                        "OutOfBoundsException",
                        b"line is past end of file".to_vec(),
                        None,
                        0,
                        span,
                    ));
                }
                let index = match operation {
                    FileObjectBuiltin::Rewind => 0,
                    FileObjectBuiltin::Advance => object
                        .property(PropertyId(3))
                        .map_err(|kind| runtime(kind, span))?
                        .as_int()
                        .expect("initialized line index")
                        .checked_add(1)
                        .expect("advance was checked before file job"),
                    FileObjectBuiltin::Seek => line.expect("seek has target line"),
                    _ => unreachable!(),
                };
                object
                    .set_property(PropertyId(3), Value::integer(index))
                    .map_err(|kind| runtime(kind, span))?;
                let value = bytes
                    .map(Value::try_bytes)
                    .transpose()
                    .map_err(|kind| runtime(kind, span))?
                    .unwrap_or(Value::NULL);
                object
                    .set_property(PropertyId(2), value)
                    .map_err(|kind| runtime(kind, span))?;
                Ok(Value::NULL)
            }
            _ => Err(runtime(RuntimeErrorKind::Unreachable, span)),
        }
    }

    fn file_open_result_value(
        &self,
        completion: &OpenCompletion,
        result: std::io::Result<std::fs::File>,
        span: Span,
    ) -> Result<Value, VmError> {
        let file = result.map_err(|error| {
            self.open_stream_error(
                std::str::from_utf8(&completion.path).expect("validated file path"),
                &error,
                span,
            )
        })?;
        let stream = Value::try_file_stream(
            completion.class,
            file,
            completion.flags.read,
            completion.flags.write,
            completion.flags.append,
        )
        .map_err(|kind| runtime(kind, span))?;
        if let Some(object) = &completion.object {
            object
                .set_property(
                    PropertyId(0),
                    Value::try_bytes(completion.path.clone())
                        .map_err(|kind| runtime(kind, span))?,
                )
                .map_err(|kind| runtime(kind, span))?;
            object
                .set_property(PropertyId(1), stream)
                .map_err(|kind| runtime(kind, span))?;
            object
                .set_property(PropertyId(2), Value::NULL)
                .map_err(|kind| runtime(kind, span))?;
            object
                .set_property(PropertyId(3), Value::integer(0))
                .map_err(|kind| runtime(kind, span))?;
            Ok(Value::NULL)
        } else {
            Ok(stream)
        }
    }

    fn directory_result_value(
        &self,
        completion: &DirectoryCompletion,
        result: Result<Vec<Vec<u8>>, DirectoryReadError>,
        span: Span,
    ) -> Result<Value, VmError> {
        let entries = result.map_err(|error| match error {
            DirectoryReadError::Open(error) => self.native_exception(
                "OpenStreamException",
                format!("cannot open directory `{}`: {error}", completion.directory).into_bytes(),
                Some(
                    completion.arguments[1]
                        .as_bytes()
                        .expect("directory path")
                        .to_vec(),
                ),
                i64::from(error.raw_os_error().unwrap_or(-1)),
                span,
            ),
            DirectoryReadError::Entry(error) => {
                self.native_exception("IoException", error.to_string().into_bytes(), None, 0, span)
            }
            DirectoryReadError::NonUtf8 => self.native_exception(
                "IoException",
                b"non-UTF-8 directory entry".to_vec(),
                None,
                0,
                span,
            ),
            DirectoryReadError::WorkerStopped => {
                runtime(RuntimeErrorKind::Io("file worker stopped".to_owned()), span)
            }
        })?;
        self.directory_builtin_with_entries(
            completion.operation,
            &completion.arguments,
            &completion.instruction,
            Some(
                entries
                    .into_iter()
                    .map(|name| String::from_utf8(name).expect("worker validated UTF-8 names"))
                    .collect(),
            ),
        )
    }

    fn recursive_directory_result_value(
        &self,
        completion: &RecursiveDirectoryCompletion,
        result: Result<Option<Vec<Vec<u8>>>, DirectoryReadError>,
        span: Span,
    ) -> Result<Value, VmError> {
        let path = std::str::from_utf8(&completion.path).expect("validated directory path");
        let entries = result.map_err(|error| match error {
            DirectoryReadError::Open(error) => self.native_exception(
                "OpenStreamException",
                format!("cannot open directory `{path}`: {error}").into_bytes(),
                Some(completion.path.clone()),
                i64::from(error.raw_os_error().unwrap_or(-1)),
                span,
            ),
            DirectoryReadError::Entry(error) => {
                self.native_exception("IoException", error.to_string().into_bytes(), None, 0, span)
            }
            DirectoryReadError::NonUtf8 => self.native_exception(
                "IoException",
                b"non-UTF-8 directory entry".to_vec(),
                None,
                0,
                span,
            ),
            DirectoryReadError::WorkerStopped => {
                runtime(RuntimeErrorKind::Io("file worker stopped".to_owned()), span)
            }
        })?;
        let child = entries.map_or(RecursiveChild::Missing, |entries| {
            RecursiveChild::Entries(
                entries
                    .into_iter()
                    .map(|name| String::from_utf8(name).expect("worker validated UTF-8 names"))
                    .collect(),
            )
        });
        self.recursive_directory_value(
            &completion.object,
            Some(&completion.info),
            &completion.instruction,
            child,
        )
    }

    fn task_error(
        &mut self,
        frames: &mut Vec<TaskFrame>,
        mut error: VmError,
    ) -> Result<(), VmError> {
        loop {
            let Some(entry) = frames.last_mut() else {
                return Err(error);
            };
            let function = &self.program.functions[entry.function.0 as usize];
            let current = entry.frame.current;
            if let VmError::Thrown { value, .. } = &error
                && catch_exception(
                    self.program,
                    function,
                    &mut entry.frame,
                    current,
                    value.clone(),
                )
            {
                let scratch = entry
                    .native_iterators
                    .first()
                    .map(|state| state.scratch().0 as usize)
                    .into_iter()
                    .chain(
                        entry
                            .iterator
                            .as_ref()
                            .map(|state| state.scratch.0 as usize),
                    )
                    .min();
                if let Some(scratch) = scratch {
                    entry.frame.registers.truncate(scratch);
                }
                entry.collection = None;
                entry.iterator = None;
                entry.native_iterators.clear();
                entry.frame.next_instruction = 0;
                return Ok(());
            }
            let mut departed = frames.pop().expect("task frame exists");
            if let Some(mut generator) = departed.generator.take() {
                generator.state.status = if generator.close_marker.is_some() {
                    GeneratorStatus::Closed
                } else {
                    GeneratorStatus::Exhausted
                };
                generator.state.key = None;
                generator.state.value = None;
                if generator.close_marker.is_some() {
                    generator.state.return_value = None;
                }
                generator.iterator.put_generator_state(generator.state);
                if let Some(marker) = generator.close_marker
                    && matches!(&error, VmError::Thrown { value, .. } if value.exception_system_code() == Ok(GENERATOR_EXIT_CODE))
                {
                    let mut failures = marker
                        .exception_suppressed()
                        .map_err(|kind| runtime(kind, departed.caller_span))?
                        .into_iter();
                    if let Some(primary) = failures.next() {
                        for failure in failures {
                            primary
                                .add_suppressed(failure)
                                .map_err(|kind| runtime(kind, departed.caller_span))?;
                        }
                        error = VmError::Thrown {
                            value: primary,
                            span: departed.caller_span,
                            trace: Vec::new(),
                        };
                    } else {
                        if let Some(parent) = frames.last_mut()
                            && let Some(destination) = departed.destination
                        {
                            parent.frame.registers[destination.0 as usize] = Some(Value::NULL);
                        }
                        return Ok(());
                    }
                }
            }
            if let Some(parent) = frames.last() {
                let caller = &self.program.functions[parent.function.0 as usize];
                match &mut error {
                    VmError::Thrown { trace, .. } => {
                        trace.push(self.stack_frame(caller, departed.caller_span));
                    }
                    VmError::Runtime(runtime) => {
                        runtime.push_frame(caller.name.clone(), departed.caller_span);
                    }
                    _ => {}
                }
            }
        }
    }

    fn finish_generator_yield(
        &self,
        frames: &mut Vec<TaskFrame>,
        key: Option<Register>,
        value: Register,
        resume: BlockId,
        span: Span,
    ) -> Result<(), VmError> {
        let entry = frames.last_mut().expect("generator frame");
        if entry
            .generator
            .as_ref()
            .expect("generator activation")
            .close_marker
            .is_some()
        {
            return Err(self.native_exception(
                "Error",
                b"generator yielded during close".to_vec(),
                None,
                0,
                span,
            ));
        }
        let explicit_key = key
            .map(|register| get_register(&entry.frame, register, span))
            .transpose()?;
        let yielded = get_register(&entry.frame, value, span)?;
        let state = &mut entry
            .generator
            .as_mut()
            .expect("generator activation")
            .state;
        let key = if let Some(key) = explicit_key {
            if let Some(number) = key.as_int()
                && state.next_key.is_some_and(|next| number >= next)
            {
                state.next_key = number.checked_add(1);
            }
            key
        } else {
            let Some(next) = state.next_key else {
                return Err(self.native_exception(
                    "ValueError",
                    b"generator automatic key overflow".to_vec(),
                    None,
                    0,
                    span,
                ));
            };
            state.next_key = next.checked_add(1);
            Value::integer(next)
        };
        entry.frame.previous = Some(entry.frame.current);
        entry.frame.current = resume;
        entry.frame.next_instruction = 0;
        let mut departed = frames.pop().expect("generator frame");
        let mut activation = departed.generator.take().expect("generator activation");
        activation.state.frame = departed.frame.into_generator();
        activation.state.key = Some(key);
        activation.state.value = Some(yielded);
        activation.state.status = GeneratorStatus::Suspended;
        activation.iterator.put_generator_state(activation.state);
        if let Some(parent) = frames.last_mut()
            && let Some(destination) = departed.destination
        {
            parent.frame.registers[destination.0 as usize] = Some(Value::NULL);
        }
        Ok(())
    }

    fn step_task(&mut self, frames: &mut Vec<TaskFrame>) -> Result<TaskOutcome, VmError> {
        loop {
            self.maximum_call_depth = self.maximum_call_depth.max(frames.len() - 1);
            self.active_functions = frames.iter().map(|entry| entry.function).collect();
            self.call_stack = frames
                .windows(2)
                .map(|pair| {
                    self.stack_frame(
                        &self.program.functions[pair[0].function.0 as usize],
                        pair[1].caller_span,
                    )
                })
                .collect();
            if frames
                .last()
                .is_some_and(|entry| entry.collection.is_some())
            {
                if let Err(error) = self.advance_collection(frames) {
                    self.task_error(frames, error)?;
                }
                continue;
            }
            if frames
                .last()
                .is_some_and(|entry| !entry.native_iterators.is_empty())
            {
                let next = match frames
                    .last()
                    .expect("native iterator frame")
                    .native_iterators
                    .last()
                {
                    Some(NativeIteratorContinuation::Filter(_)) => self.advance_filter(frames),
                    Some(NativeIteratorContinuation::Limit(_)) => self.advance_limit(frames),
                    Some(NativeIteratorContinuation::Append(_)) => self.advance_append(frames),
                    Some(NativeIteratorContinuation::Caching(_)) => self.advance_caching(frames),
                    Some(NativeIteratorContinuation::Infinite(_)) => self.advance_infinite(frames),
                    Some(NativeIteratorContinuation::Regex(_)) => self.advance_regex(frames),
                    Some(NativeIteratorContinuation::Walk(_)) => self.advance_walk(frames),
                    Some(NativeIteratorContinuation::LimitSeek(_)) => {
                        self.advance_limit_seek(frames)
                    }
                    None => unreachable!(),
                };
                match next {
                    Ok(CallOutcome::Continue) => continue,
                    Ok(CallOutcome::Suspend(wait)) => return Ok(TaskOutcome::Suspended(wait)),
                    Err(error) => {
                        self.task_error(frames, error)?;
                        continue;
                    }
                }
            }
            if frames.last().is_some_and(|entry| entry.iterator.is_some()) {
                match self.advance_iterator(frames) {
                    Ok(CallOutcome::Continue) => continue,
                    Ok(CallOutcome::Suspend(wait)) => return Ok(TaskOutcome::Suspended(wait)),
                    Err(error) => {
                        self.task_error(frames, error)?;
                        continue;
                    }
                }
            }
            let depth = frames.len();
            let entry = frames.last_mut().expect("running coroutine has a frame");
            let function = &self.program.functions[entry.function.0 as usize];
            let block = &function.blocks[entry.frame.current.0 as usize];
            if let Some(instruction) = block.instructions.get(entry.frame.next_instruction) {
                if let Err(error) = self.tick(instruction.span) {
                    self.task_error(frames, error)?;
                    continue;
                }
                if matches!(instruction.kind, InstructionKind::NewDynamic { .. }) {
                    if let Err(error) = self.task_new_dynamic(frames, instruction) {
                        self.task_error(frames, error)?;
                    }
                    continue;
                }
                let call = match &instruction.kind {
                    InstructionKind::Call { callee, arguments } => Some((
                        *callee,
                        arguments
                            .iter()
                            .map(|register| get_register(&entry.frame, *register, instruction.span))
                            .collect::<Result<Vec<_>, _>>()?,
                        None,
                    )),
                    InstructionKind::CallValue { callee, arguments } => {
                        let callback = get_register(&entry.frame, *callee, instruction.span)?;
                        let (target, captures) = callback.closure_parts().ok_or_else(|| {
                            self.async_error("invalid callable", instruction.span)
                        })?;
                        let mut values = captures.to_vec();
                        values.extend(
                            arguments
                                .iter()
                                .map(|register| {
                                    get_register(&entry.frame, *register, instruction.span)
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        Some((Callee::Function(target), values, None))
                    }
                    InstructionKind::DirectMethod {
                        callee,
                        arguments,
                        called_class,
                    } => {
                        let values = arguments
                            .iter()
                            .map(|register| get_register(&entry.frame, *register, instruction.span))
                            .collect::<Result<Vec<_>, _>>()?;
                        let called = match called_class {
                            CalledClass::Explicit(class) => Some(*class),
                            CalledClass::Forwarded => entry.frame.called_class,
                            CalledClass::Receiver => values.first().and_then(Value::class_id),
                        };
                        Some((*callee, values, called))
                    }
                    InstructionKind::VirtualMethod {
                        receiver,
                        slot,
                        arguments,
                    } => {
                        let receiver = get_register(&entry.frame, *receiver, instruction.span)?;
                        let class = receiver.class_id().ok_or_else(|| {
                            self.async_error("invalid method receiver", instruction.span)
                        })?;
                        let (callee, static_method) =
                            self.dispatch_target(class, *slot, instruction.span)?;
                        let mut values = arguments
                            .iter()
                            .map(|register| get_register(&entry.frame, *register, instruction.span))
                            .collect::<Result<Vec<_>, _>>()?;
                        if !static_method {
                            values.insert(0, receiver);
                        }
                        Some((callee, values, Some(class)))
                    }
                    InstructionKind::LateStaticMethod {
                        receiver,
                        slot,
                        arguments,
                    } => {
                        let class = entry.frame.called_class.ok_or_else(|| {
                            self.async_error("missing late static class", instruction.span)
                        })?;
                        let (callee, static_method) =
                            self.dispatch_target(class, *slot, instruction.span)?;
                        let mut values = arguments
                            .iter()
                            .map(|register| get_register(&entry.frame, *register, instruction.span))
                            .collect::<Result<Vec<_>, _>>()?;
                        if !static_method {
                            let receiver = receiver
                                .and_then(|register| {
                                    get_register(&entry.frame, register, instruction.span).ok()
                                })
                                .ok_or_else(|| {
                                    self.async_error("missing method receiver", instruction.span)
                                })?;
                            values.insert(0, receiver);
                        }
                        Some((callee, values, Some(class)))
                    }
                    _ => None,
                };
                let result = if let Some((callee, arguments, called_class)) = call {
                    self.task_call(frames, callee, arguments, called_class, instruction)
                } else {
                    self.execute_instruction(function, &mut entry.frame, instruction, depth)
                        .map(|value| {
                            if let Some(destination) = instruction.destination {
                                entry.frame.registers[destination.0 as usize] =
                                    Some(value.unwrap_or(Value::NULL));
                            }
                            entry.frame.next_instruction += 1;
                            CallOutcome::Continue
                        })
                };
                match result {
                    Ok(CallOutcome::Continue) => continue,
                    Ok(CallOutcome::Suspend(wait)) => return Ok(TaskOutcome::Suspended(wait)),
                    Err(error) => {
                        self.task_error(frames, error)?;
                        continue;
                    }
                }
            }
            if let Err(error) = self.tick(function.span) {
                self.task_error(frames, error)?;
                continue;
            }
            if let Terminator::Yield { key, value, resume } = block.terminator
                && entry.generator.is_some()
            {
                if let Err(error) =
                    self.finish_generator_yield(frames, key, value, resume, function.span)
                {
                    self.task_error(frames, error)?;
                }
                continue;
            }
            let previous = entry.frame.current;
            let result: Result<Option<Value>, VmError> = match block.terminator {
                Terminator::Jump(target) => {
                    entry.frame.previous = Some(previous);
                    entry.frame.current = target;
                    entry.frame.next_instruction = 0;
                    Ok(None)
                }
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => {
                    let value = get_register(&entry.frame, condition, function.span)?;
                    let condition = value.as_bool().ok_or_else(|| {
                        runtime(
                            thp_runtime::RuntimeErrorKind::TypeError(
                                "branch condition is not bool".to_owned(),
                            ),
                            function.span,
                        )
                    })?;
                    entry.frame.previous = Some(previous);
                    entry.frame.current = if condition { then_block } else { else_block };
                    entry.frame.next_instruction = 0;
                    Ok(None)
                }
                Terminator::Return(value) => {
                    Ok(Some(value.map_or(Ok(Value::NULL), |register| {
                        get_register(&entry.frame, register, function.span)
                    })?))
                }
                Terminator::Throw(value) => Err(VmError::Thrown {
                    value: get_register(&entry.frame, value, function.span)?,
                    span: function.span,
                    trace: Vec::new(),
                }),
                Terminator::Yield { .. } | Terminator::Unreachable => Err(runtime(
                    thp_runtime::RuntimeErrorKind::Unreachable,
                    function.span,
                )),
            };
            match result {
                Ok(None) => {}
                Ok(Some(value)) => {
                    let mut departed = frames.pop().expect("task frame exists");
                    if let Some(mut generator) = departed.generator.take() {
                        if generator.close_marker.is_some() {
                            generator.state.return_value = None;
                            generator.state.status = GeneratorStatus::Closed;
                        } else {
                            generator.state.return_value = Some(value);
                            generator.state.status = GeneratorStatus::Exhausted;
                        }
                        generator.state.key = None;
                        generator.state.value = None;
                        generator.iterator.put_generator_state(generator.state);
                        if let Some(parent) = frames.last_mut()
                            && let Some(destination) = departed.destination
                        {
                            parent.frame.registers[destination.0 as usize] = Some(Value::NULL);
                        }
                        continue;
                    }
                    if departed.validate_countable && value.as_int().is_none_or(|count| count < 0) {
                        let error = self.native_exception(
                            "UnexpectedValueException",
                            b"Countable::count() must return a non-negative int".to_vec(),
                            None,
                            0,
                            departed.caller_span,
                        );
                        self.task_error(frames, error)?;
                        continue;
                    }
                    if departed.reflection_return.as_ref().is_some_and(|expected| {
                        *expected != Type::Void && !value_matches(self.program, &value, expected)
                    }) {
                        let error = self.reflection_error(
                            "invoked callable returned a value of the wrong type",
                            departed.caller_span,
                        );
                        self.task_error(frames, error)?;
                        continue;
                    }
                    let value = departed.return_override.unwrap_or(value);
                    if let Some(parent) = frames.last_mut() {
                        if let Some(collection) = parent.collection.as_mut()
                            && collection.pending
                        {
                            let span = collection.instruction.span;
                            if let Err(kind) = collection.receive(value) {
                                self.task_error(frames, runtime(kind, span))?;
                            }
                            continue;
                        }
                        if let Some(destination) = departed.destination {
                            parent.frame.registers[destination.0 as usize] = Some(value);
                        }
                    } else {
                        return Ok(TaskOutcome::Completed(value));
                    }
                }
                Err(error) => {
                    self.task_error(frames, error)?;
                }
            }
        }
    }

    fn await_source(
        &mut self,
        target: &Value,
        cancellation: &Value,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        let span = instruction.span;
        let id = self.completion_id(target, depth, calling_function, instruction)?;
        if self.async_runtime.sources[id - 1].result.is_some() {
            return self.read_result(id, span);
        }
        self.require_scheduler(span)?;
        self.check_async_cancellation(span)?;
        let cancellation_id = if cancellation.is_null() {
            None
        } else {
            Some(self.completion_id(cancellation, depth, calling_function, instruction)?)
        };
        self.async_runtime.root_wait = Some((id, cancellation_id));
        self.async_runtime.root_winner = cancellation_id
            .filter(|cancel| self.async_runtime.sources[*cancel - 1].result.is_some())
            .map(|_| false);
        if let Some(current) = self.async_runtime.current {
            self.async_runtime.sources[current - 1].suspended = true;
        }
        loop {
            self.fire_timers(depth, calling_function, instruction)?;
            if self.async_runtime.root_winner == Some(true)
                || (self.async_runtime.root_winner.is_none()
                    && self.async_runtime.sources[id - 1].result.is_some())
            {
                break;
            }
            if self.async_runtime.root_winner == Some(false) {
                let cancel_id = cancellation_id.expect("cancellation winner has source");
                let cause = self.cancellation_cause(Some(cancel_id));
                let error = self.native_exception(
                    "WaitCancelledException",
                    b"wait cancelled".to_vec(),
                    None,
                    0,
                    span,
                );
                let VmError::Thrown { value, .. } = &error else {
                    return Err(error);
                };
                if let Some(cause) = cause {
                    value
                        .append_previous(cause)
                        .map_err(|kind| runtime(kind, span))?;
                }
                self.async_runtime.root_wait = None;
                self.async_runtime.root_winner = None;
                self.release_external_if_unused(id, depth, calling_function, instruction)?;
                if let Some(cancel_id) = cancellation_id {
                    self.release_external_if_unused(
                        cancel_id,
                        depth,
                        calling_function,
                        instruction,
                    )?;
                }
                return Err(error);
            }
            if !self.run_ready(depth, calling_function, instruction)? {
                let next = self.next_wakeup_deadline();
                if let Some(deadline) = next {
                    std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                } else {
                    return Err(self.native_exception(
                        "AsyncDeadlockException",
                        b"no runnable coroutine or wakeup".to_vec(),
                        None,
                        0,
                        span,
                    ));
                }
            }
            self.tick(span)?;
            self.check_async_cancellation(span)?;
        }
        if let Some(current) = self.async_runtime.current {
            self.async_runtime.sources[current - 1].suspended = false;
        }
        self.async_runtime.root_wait = None;
        self.async_runtime.root_winner = None;
        self.release_external_if_unused(id, depth, calling_function, instruction)?;
        if let Some(cancel_id) = cancellation_id {
            self.release_external_if_unused(cancel_id, depth, calling_function, instruction)?;
        }
        self.read_result(id, span)
    }

    pub(super) fn drain_async(&mut self, span: Span) -> Result<(), VmError> {
        let function = &self.program.functions[self.program.entry.0 as usize];
        let instruction = Instruction {
            kind: thp_bytecode::InstructionKind::Constant(thp_mir::Constant::Null),
            destination: None,
            ty: None,
            span,
        };
        while self.async_runtime.sources.iter().any(|source| {
            matches!(source.kind, SourceKind::Coroutine { .. }) && source.result.is_none()
        }) {
            if !self.run_ready(0, function, &instruction)? {
                let next = self.next_wakeup_deadline();
                if let Some(deadline) = next {
                    std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                } else {
                    return Err(self.native_exception(
                        "AsyncDeadlockException",
                        b"no runnable coroutine or wakeup".to_vec(),
                        None,
                        0,
                        span,
                    ));
                }
            }
            self.tick(span)?;
        }
        let mut failures = self.async_runtime.sources.iter().filter_map(|source| {
            if matches!(source.kind, SourceKind::Coroutine { .. }) && !source.observed {
                source
                    .result
                    .as_ref()
                    .and_then(|result| result.as_ref().err())
                    .map(|failure| (failure.clone(), source.failure_trace.clone()))
            } else {
                None
            }
        });
        if let Some((primary, trace)) = failures.next() {
            for (failure, _) in failures {
                primary
                    .add_suppressed(failure)
                    .map_err(|kind| runtime(kind, span))?;
            }
            return Err(VmError::Thrown {
                value: primary,
                span,
                trace,
            });
        }
        Ok(())
    }
}
