//! The reference interpreter for verified THP bytecode.

#![allow(clippy::float_cmp, clippy::too_many_lines)]

use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use thp_bytecode::{
    Function, Instruction, InstructionKind, Program, Terminator, VerificationError, verify,
};
use thp_diagnostics::Span;
use thp_hir::{Builtin, CalledClass, Callee, ClassId, FunctionId, MethodSlot, Type};
use thp_mir::{BlockId, Constant, Register};
use thp_runtime::{
    HeapStats, RequestHeap, RequestInput, RuntimeError, RuntimeErrorKind, StackFrame, Value,
};
use thp_syntax::{BinaryOp, UnaryOp};

#[derive(Clone, Copy, Debug, Default)]
pub struct Limits {
    /// Stops execution after this many bytecode instructions and terminators.
    /// `None` means no instruction limit.
    pub max_instructions: Option<u64>,
    /// Stops execution after this wall-clock duration. The limit is checked
    /// cooperatively between bytecode instructions and terminators.
    /// `None` means no execution-time limit.
    pub max_execution: Option<Duration>,
    /// Maximum bytes owned by the request's managed THP heap.
    pub max_heap_bytes: Option<usize>,
    /// Maximum bytes consumable from the SAPI request body/stdin.
    pub max_input_bytes: Option<u64>,
    /// Maximum elapsed time while the request body remains open.
    pub max_input_time: Option<Duration>,
    /// Maximum logical THP function-call depth.
    pub max_stack_depth: Option<usize>,
    /// Maximum number of distinct open THP stream cells.
    pub max_open_handles: Option<usize>,
}

/// Request-local VM state that must not be process-global.
#[derive(Clone, Debug)]
pub struct ExecutionContext {
    pub limits: Limits,
    /// Base directory used to resolve relative filesystem operations.
    pub filesystem_base: PathBuf,
    /// Shared binary body exposed through `thp:/input`.
    pub request_input: RequestInput,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            filesystem_base: PathBuf::new(),
            request_input: RequestInput::empty(),
        }
    }
}

#[derive(Debug)]
pub struct Execution {
    pub result: Value,
    pub output: Vec<u8>,
    pub instructions: u64,
    pub maximum_call_depth: usize,
    pub output_bytes: u64,
    pub heap: HeapStats,
    _request_heap: Option<RequestHeap>,
}

#[derive(Debug)]
pub struct ExecutionFailure {
    pub error: VmError,
    pub output: Vec<u8>,
    pub instructions: u64,
    pub maximum_call_depth: usize,
    pub output_bytes: u64,
    pub heap: HeapStats,
    _request_heap: Option<RequestHeap>,
}

#[derive(Debug)]
pub struct StreamingExecution {
    pub result: Value,
    pub instructions: u64,
    pub maximum_call_depth: usize,
    pub output_bytes: u64,
    pub heap: HeapStats,
    request_heap: Option<RequestHeap>,
}

#[derive(Debug)]
pub struct StreamingExecutionFailure {
    pub error: VmError,
    pub instructions: u64,
    pub maximum_call_depth: usize,
    pub output_bytes: u64,
    pub heap: HeapStats,
    request_heap: Option<RequestHeap>,
}

#[derive(Debug)]
pub enum VmError {
    Verification(VerificationError),
    Runtime(RuntimeError),
    Thrown {
        value: Value,
        span: Span,
        trace: Vec<StackFrame>,
    },
    InstructionLimit {
        limit: u64,
        span: Span,
    },
    ExecutionTimeLimit {
        limit: Duration,
        span: Span,
    },
    StackDepthLimit {
        limit: usize,
        span: Span,
    },
    OutputIo {
        message: String,
        span: Span,
    },
}

impl fmt::Display for VmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Verification(error) => error.fmt(formatter),
            Self::Runtime(error) => error.fmt(formatter),
            Self::Thrown { value, .. } => write!(
                formatter,
                "uncaught object of runtime class #{}",
                value.class_id().map_or(u32::MAX, |class| class.0)
            ),
            Self::InstructionLimit { limit, .. } => {
                write!(
                    formatter,
                    "execution exceeded the {limit} instruction limit"
                )
            }
            Self::ExecutionTimeLimit { limit, .. } => {
                write!(
                    formatter,
                    "execution exceeded the {} second time limit",
                    limit.as_secs()
                )
            }
            Self::StackDepthLimit { limit, .. } => {
                write!(
                    formatter,
                    "execution exceeded the {limit} frame stack limit"
                )
            }
            Self::OutputIo { message, .. } => {
                write!(formatter, "host output failed: {message}")
            }
        }
    }
}

impl std::error::Error for VmError {}

/// Verifies and executes a bytecode program with the reference VM.
///
/// # Errors
///
/// Returns verification, runtime, or instruction-limit failures without
/// printing or terminating the host process.
pub fn execute(program: &Program, limits: Limits) -> Result<Execution, VmError> {
    execute_captured(
        program,
        &ExecutionContext {
            limits,
            filesystem_base: PathBuf::new(),
            request_input: RequestInput::empty(),
        },
    )
    .map_err(|failure| failure.error)
}

/// Verifies and executes bytecode while retaining output and counters on
/// failure.
///
/// # Errors
///
/// Returns verification, runtime, instruction-limit, or time-limit failures
/// together with output produced before the failure.
#[allow(clippy::result_large_err)]
pub fn execute_captured(
    program: &Program,
    context: &ExecutionContext,
) -> Result<Execution, ExecutionFailure> {
    let mut output = FallibleCapture::default();
    match execute_to(program, context, &mut output) {
        Ok(execution) => Ok(Execution {
            result: execution.result,
            output: output.bytes,
            instructions: execution.instructions,
            maximum_call_depth: execution.maximum_call_depth,
            output_bytes: execution.output_bytes,
            heap: execution.heap,
            _request_heap: execution.request_heap,
        }),
        Err(failure) => Err(ExecutionFailure {
            error: failure.error,
            output: output.bytes,
            instructions: failure.instructions,
            maximum_call_depth: failure.maximum_call_depth,
            output_bytes: failure.output_bytes,
            heap: failure.heap,
            _request_heap: failure.request_heap,
        }),
    }
}

/// Verifies and executes bytecode while streaming output to the host.
///
/// The sink is synchronous: successful writes provide natural backpressure.
/// A sink error is a non-catchable request failure and already accepted bytes
/// are not rolled back.
///
/// # Errors
///
/// Returns verification, runtime, policy, time, or output-sink failures with
/// request counters and heap statistics.
#[allow(clippy::result_large_err)]
pub fn execute_to(
    program: &Program,
    context: &ExecutionContext,
    output: &mut dyn Write,
) -> Result<StreamingExecution, StreamingExecutionFailure> {
    if let Err(error) = verify(program) {
        return Err(StreamingExecutionFailure {
            error: VmError::Verification(error),
            instructions: 0,
            maximum_call_depth: 0,
            output_bytes: 0,
            heap: HeapStats::default(),
            request_heap: None,
        });
    }
    if let Err(kind) = context.request_input.apply_limits(
        context.limits.max_input_bytes,
        context.limits.max_input_time,
    ) {
        return Err(StreamingExecutionFailure {
            error: runtime(kind, program.functions[program.entry.0 as usize].span),
            instructions: 0,
            maximum_call_depth: 0,
            output_bytes: 0,
            heap: HeapStats::default(),
            request_heap: None,
        });
    }
    let request_heap = match RequestHeap::new(
        context.limits.max_heap_bytes,
        context.limits.max_open_handles,
    ) {
        Ok(heap) => heap,
        Err(kind) => {
            return Err(StreamingExecutionFailure {
                error: runtime(kind, program.functions[program.entry.0 as usize].span),
                instructions: 0,
                maximum_call_depth: 0,
                output_bytes: 0,
                heap: HeapStats::default(),
                request_heap: None,
            });
        }
    };
    let active_heap = request_heap.activate();
    let mut state = ExecutionState {
        program,
        output,
        output_bytes: 0,
        instructions: 0,
        maximum_call_depth: 0,
        limits: context.limits,
        filesystem_base: context.filesystem_base.clone(),
        request_input: context.request_input.clone(),
        request_input_stream: None,
        request_heap: &request_heap,
        started: Instant::now(),
    };
    let result = match state.execute_function(program.entry, Vec::new(), 1, None) {
        Ok(result) => Ok(result),
        Err(VmError::Thrown { value, span, trace }) => {
            let class = value
                .class_id()
                .and_then(|id| program.classes.get(id.0 as usize))
                .map_or_else(|| "<non-exception>".to_owned(), |class| class.name.clone());
            let message = value
                .exception_message()
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .unwrap_or_default();
            let mut error =
                RuntimeError::new(RuntimeErrorKind::UncaughtException { class, message }, span);
            error.trace = trace;
            Err(VmError::Runtime(error))
        }
        Err(error) => Err(error),
    };
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            let instructions = state.instructions;
            let maximum_call_depth = state.maximum_call_depth;
            let output_bytes = state.output_bytes;
            drop(state);
            request_heap.collect_cycles();
            let heap = request_heap.stats();
            drop(active_heap);
            return Err(StreamingExecutionFailure {
                error,
                instructions,
                maximum_call_depth,
                output_bytes,
                heap,
                request_heap: Some(request_heap),
            });
        }
    };
    let instructions = state.instructions;
    let maximum_call_depth = state.maximum_call_depth;
    let output_bytes = state.output_bytes;
    drop(state);
    request_heap.collect_cycles();
    let heap = request_heap.stats();
    drop(active_heap);
    Ok(StreamingExecution {
        result,
        instructions,
        maximum_call_depth,
        output_bytes,
        heap,
        request_heap: Some(request_heap),
    })
}

#[derive(Default)]
struct FallibleCapture {
    bytes: Vec<u8>,
}

impl Write for FallibleCapture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes
            .try_reserve(bytes.len())
            .map_err(|_| io::Error::other("captured output allocation failed"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct ExecutionState<'program, 'output> {
    program: &'program Program,
    output: &'output mut dyn Write,
    output_bytes: u64,
    instructions: u64,
    maximum_call_depth: usize,
    limits: Limits,
    filesystem_base: PathBuf,
    request_input: RequestInput,
    request_input_stream: Option<Value>,
    request_heap: &'output RequestHeap,
    started: Instant,
}

struct Frame {
    locals: Vec<Option<Value>>,
    registers: Vec<Option<Value>>,
    current: BlockId,
    previous: Option<BlockId>,
    called_class: Option<ClassId>,
}

impl ExecutionState<'_, '_> {
    fn execute_function(
        &mut self,
        id: FunctionId,
        arguments: Vec<Value>,
        depth: usize,
        called_class: Option<ClassId>,
    ) -> Result<Value, VmError> {
        if self
            .limits
            .max_stack_depth
            .is_some_and(|limit| depth > limit)
        {
            return Err(VmError::StackDepthLimit {
                limit: self.limits.max_stack_depth.expect("limit exists"),
                span: self.program.functions[id.0 as usize].span,
            });
        }
        self.maximum_call_depth = self.maximum_call_depth.max(depth);
        let function = &self.program.functions[id.0 as usize];
        let mut frame = Frame {
            locals: std::iter::repeat_with(|| None)
                .take(function.local_types.len())
                .collect(),
            registers: std::iter::repeat_with(|| None)
                .take(function.register_types.len())
                .collect(),
            current: function.entry,
            previous: None,
            called_class,
        };
        for (parameter, argument) in function.parameters.iter().zip(arguments) {
            frame.locals[parameter.0 as usize] = Some(argument);
        }

        'execution: loop {
            let block = &function.blocks[frame.current.0 as usize];
            for instruction in &block.instructions {
                self.tick(instruction.span)?;
                let value = match self.execute_instruction(function, &mut frame, instruction, depth)
                {
                    Ok(value) => value,
                    Err(VmError::Thrown { value, span, trace }) => {
                        if catch_exception(
                            self.program,
                            function,
                            &mut frame,
                            block.id,
                            value.clone(),
                        ) {
                            continue 'execution;
                        }
                        return Err(VmError::Thrown { value, span, trace });
                    }
                    Err(error) => return Err(error),
                };
                if let Some(destination) = instruction.destination {
                    frame.registers[destination.0 as usize] = Some(value.unwrap_or(Value::NULL));
                }
            }
            self.tick(function.span)?;
            let previous = frame.current;
            match block.terminator {
                Terminator::Jump(target) => {
                    frame.previous = Some(previous);
                    frame.current = target;
                }
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => {
                    let condition = get_register(&frame, condition, function.span)?;
                    let Some(condition) = condition.as_bool() else {
                        return Err(runtime(
                            RuntimeErrorKind::TypeError(
                                "bytecode branch condition is not bool".to_owned(),
                            ),
                            function.span,
                        ));
                    };
                    frame.previous = Some(previous);
                    frame.current = if condition { then_block } else { else_block };
                }
                Terminator::Return(value) => {
                    return value.map_or(Ok(Value::NULL), |register| {
                        get_register(&frame, register, function.span)
                    });
                }
                Terminator::Throw(value) => {
                    let value = get_register(&frame, value, function.span)?;
                    if catch_exception(self.program, function, &mut frame, block.id, value.clone())
                    {
                        continue 'execution;
                    }
                    return Err(VmError::Thrown {
                        value,
                        span: function.span,
                        trace: Vec::new(),
                    });
                }
                Terminator::Unreachable => {
                    return Err(runtime(RuntimeErrorKind::Unreachable, function.span));
                }
            }
        }
    }

    fn execute_instruction(
        &mut self,
        function: &Function,
        frame: &mut Frame,
        instruction: &Instruction,
        depth: usize,
    ) -> Result<Option<Value>, VmError> {
        let value = match &instruction.kind {
            InstructionKind::Constant(constant) => {
                Some(constant_value(constant, instruction.span)?)
            }
            InstructionKind::LoadLocal(local) => {
                Some(frame.locals[local.0 as usize].clone().ok_or_else(|| {
                    runtime(
                        RuntimeErrorKind::UninitializedLocal(local.0),
                        instruction.span,
                    )
                })?)
            }
            InstructionKind::StoreLocal { local, value } => {
                frame.locals[local.0 as usize] =
                    Some(get_register(frame, *value, instruction.span)?);
                None
            }
            InstructionKind::Unary { op, operand } => {
                let operand = get_register(frame, *operand, instruction.span)?;
                Some(execute_unary(*op, &operand, instruction.span)?)
            }
            InstructionKind::Binary { op, left, right } => {
                let left = get_register(frame, *left, instruction.span)?;
                let right = get_register(frame, *right, instruction.span)?;
                Some(execute_binary(*op, &left, &right, instruction.span)?)
            }
            InstructionKind::IsNull(register) => Some(Value::bool(
                get_register(frame, *register, instruction.span)?.is_null(),
            )),
            InstructionKind::Vector(registers) => {
                let values = registers
                    .iter()
                    .map(|register| get_register(frame, *register, instruction.span))
                    .collect::<Result<Vec<_>, _>>()?;
                let Type::Vector(element) = instruction.ty.as_ref().expect("verified result type")
                else {
                    unreachable!("verifier ensures vector result type")
                };
                Some(
                    Value::try_vector(element.as_ref().clone(), values)
                        .map_err(|kind| runtime(kind, instruction.span))?,
                )
            }
            InstructionKind::Map(entries) => {
                let entries = entries
                    .iter()
                    .map(|(key, value)| {
                        Ok((
                            get_register(frame, *key, instruction.span)?,
                            get_register(frame, *value, instruction.span)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, VmError>>()?;
                let Type::Map(key, value) = instruction.ty.as_ref().expect("verified result type")
                else {
                    unreachable!("verifier ensures map result type")
                };
                Some(
                    Value::try_map(key.as_ref().clone(), value.as_ref().clone(), entries)
                        .map_err(|kind| runtime(kind, instruction.span))?,
                )
            }
            InstructionKind::Index { collection, index } => {
                let collection = get_register(frame, *collection, instruction.span)?;
                let index = get_register(frame, *index, instruction.span)?;
                Some(
                    collection
                        .index(&index)
                        .map_err(|kind| runtime(kind, instruction.span))?,
                )
            }
            InstructionKind::CollectionLen(collection) => {
                let collection = get_register(frame, *collection, instruction.span)?;
                let length = collection
                    .collection_len()
                    .map_err(|kind| runtime(kind, instruction.span))?;
                Some(Value::integer(i64::try_from(length).map_err(|_| {
                    runtime(
                        RuntimeErrorKind::Arithmetic(
                            "collection length exceeds the signed 64-bit range".to_owned(),
                        ),
                        instruction.span,
                    )
                })?))
            }
            InstructionKind::CollectionKeyAt { collection, offset } => {
                let collection = get_register(frame, *collection, instruction.span)?;
                let offset = collection_offset(frame, *offset, instruction.span)?;
                Some(
                    collection
                        .collection_key_at(offset)
                        .map_err(|kind| runtime(kind, instruction.span))?,
                )
            }
            InstructionKind::CollectionValueAt { collection, offset } => {
                let collection = get_register(frame, *collection, instruction.span)?;
                let offset = collection_offset(frame, *offset, instruction.span)?;
                Some(
                    collection
                        .collection_value_at(offset)
                        .map_err(|kind| runtime(kind, instruction.span))?,
                )
            }
            InstructionKind::SetIndex {
                collection,
                index,
                value,
            } => {
                let mut collection = get_register(frame, *collection, instruction.span)?;
                let index = get_register(frame, *index, instruction.span)?;
                let value = get_register(frame, *value, instruction.span)?;
                collection
                    .set_index(&index, value)
                    .map_err(|kind| runtime(kind, instruction.span))?;
                Some(collection)
            }
            InstructionKind::Call { callee, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|register| get_register(frame, *register, instruction.span))
                    .collect::<Result<Vec<_>, _>>()?;
                Some(self.invoke_callee(*callee, arguments, depth, None, function, instruction)?)
            }
            InstructionKind::DirectMethod {
                callee,
                arguments,
                called_class,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|register| get_register(frame, *register, instruction.span))
                    .collect::<Result<Vec<_>, _>>()?;
                let called_class = match called_class {
                    CalledClass::Explicit(class) => Some(*class),
                    CalledClass::Forwarded => frame.called_class,
                    CalledClass::Receiver => arguments.first().and_then(Value::class_id),
                };
                Some(self.invoke_callee(
                    *callee,
                    arguments,
                    depth,
                    called_class,
                    function,
                    instruction,
                )?)
            }
            InstructionKind::VirtualMethod {
                receiver,
                slot,
                arguments,
            } => {
                let receiver = get_register(frame, *receiver, instruction.span)?;
                let actual = receiver
                    .class_id()
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, instruction.span))?;
                let (callee, static_method) =
                    self.dispatch_target(actual, *slot, instruction.span)?;
                let mut arguments = arguments
                    .iter()
                    .map(|register| get_register(frame, *register, instruction.span))
                    .collect::<Result<Vec<_>, _>>()?;
                if !static_method {
                    arguments.insert(0, receiver);
                }
                Some(self.invoke_callee(
                    callee,
                    arguments,
                    depth,
                    Some(actual),
                    function,
                    instruction,
                )?)
            }
            InstructionKind::LateStaticMethod {
                receiver,
                slot,
                arguments,
            } => {
                let called_class = frame
                    .called_class
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, instruction.span))?;
                let (callee, static_method) =
                    self.dispatch_target(called_class, *slot, instruction.span)?;
                let mut arguments = arguments
                    .iter()
                    .map(|register| get_register(frame, *register, instruction.span))
                    .collect::<Result<Vec<_>, _>>()?;
                if !static_method {
                    let receiver = receiver
                        .map(|receiver| get_register(frame, receiver, instruction.span))
                        .transpose()?
                        .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, instruction.span))?;
                    arguments.insert(0, receiver);
                }
                Some(self.invoke_callee(
                    callee,
                    arguments,
                    depth,
                    Some(called_class),
                    function,
                    instruction,
                )?)
            }
            InstructionKind::NewObject(class) => {
                let class = &self.program.classes[class.0 as usize];
                Some(
                    if is_instance_of_name(self.program, class.id, "Throwable") {
                        Value::try_throwable_object(class.id, class.properties.len())
                    } else {
                        Value::try_object(class.id, class.properties.len())
                    }
                    .map_err(|kind| runtime(kind, instruction.span))?,
                )
            }
            InstructionKind::GetProperty { object, property } => {
                let object = get_register(frame, *object, instruction.span)?;
                Some(
                    object
                        .property(*property)
                        .map_err(|kind| runtime(kind, instruction.span))?,
                )
            }
            InstructionKind::SetProperty {
                object,
                property,
                value,
            }
            | InstructionKind::InitializeProperty {
                object,
                property,
                value,
            } => {
                let object = get_register(frame, *object, instruction.span)?;
                let value = get_register(frame, *value, instruction.span)?;
                object
                    .set_property(*property, value)
                    .map_err(|kind| runtime(kind, instruction.span))?;
                None
            }
            InstructionKind::InstanceOf { value, class } => {
                let value = get_register(frame, *value, instruction.span)?;
                let matches = value
                    .class_id()
                    .is_some_and(|actual| is_instance_of(self.program, actual, *class));
                Some(Value::bool(matches))
            }
            InstructionKind::AddSuppressed {
                primary,
                suppressed,
            } => {
                let primary = get_register(frame, *primary, instruction.span)?;
                let suppressed = get_register(frame, *suppressed, instruction.span)?;
                primary
                    .add_suppressed(suppressed)
                    .map_err(|kind| runtime(kind, instruction.span))?;
                None
            }
            InstructionKind::ChainPrevious {
                replacement,
                previous,
            } => {
                let replacement = get_register(frame, *replacement, instruction.span)?;
                let previous = get_register(frame, *previous, instruction.span)?;
                replacement
                    .append_previous(previous)
                    .map_err(|kind| runtime(kind, instruction.span))?;
                None
            }
            InstructionKind::RaiseUnhandledMatch(value) => {
                let value = get_register(frame, *value, instruction.span)?;
                return Err(self.native_exception(
                    "UnhandledMatchError",
                    format!("no match arm handled {}", describe_match_subject(&value)).into_bytes(),
                    None,
                    0,
                    instruction.span,
                ));
            }
            InstructionKind::Phi(inputs) => {
                let Some(previous) = frame.previous else {
                    return Err(runtime(RuntimeErrorKind::Unreachable, instruction.span));
                };
                let Some((_, register)) = inputs
                    .iter()
                    .find(|(predecessor, _)| *predecessor == previous)
                else {
                    return Err(runtime(RuntimeErrorKind::Unreachable, instruction.span));
                };
                Some(get_register(frame, *register, instruction.span)?)
            }
            InstructionKind::Print(register) => {
                let value = get_register(frame, *register, instruction.span)?;
                let bytes = value
                    .output_bytes()
                    .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, instruction.span))?;
                self.write_output(&bytes, instruction.span)?;
                None
            }
        };
        Ok(value)
    }

    fn invoke_callee(
        &mut self,
        callable: Callee,
        arguments: Vec<Value>,
        depth: usize,
        called_class: Option<ClassId>,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        match callable {
            Callee::Function(target) => {
                match self.execute_function(target, arguments, depth + 1, called_class) {
                    Ok(value) => Ok(value),
                    Err(VmError::Runtime(mut error)) => {
                        error.push_frame(calling_function.name.clone(), instruction.span);
                        Err(VmError::Runtime(error))
                    }
                    Err(VmError::Thrown {
                        value,
                        span,
                        mut trace,
                    }) => {
                        trace.push(StackFrame {
                            function: calling_function.name.clone(),
                            span: instruction.span,
                        });
                        Err(VmError::Thrown { value, span, trace })
                    }
                    Err(error) => Err(error),
                }
            }
            Callee::Builtin(builtin) => self.execute_builtin(builtin, arguments, instruction),
        }
    }

    fn dispatch_target(
        &self,
        class: ClassId,
        slot: MethodSlot,
        span: Span,
    ) -> Result<(Callee, bool), VmError> {
        let class = &self.program.classes[class.0 as usize];
        let callee = class
            .dispatch
            .get(slot.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let method = class
            .methods
            .iter()
            .find(|method| method.slot == slot)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        Ok((callee, method.static_method))
    }

    fn execute_builtin(
        &mut self,
        builtin: Builtin,
        arguments: Vec<Value>,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        match builtin {
            Builtin::Count => {
                let count = arguments[0].count().ok_or_else(|| {
                    runtime(
                        RuntimeErrorKind::TypeError(format!(
                            "cannot count {}",
                            arguments[0].type_name()
                        )),
                        instruction.span,
                    )
                })?;
                Ok(Value::integer(i64::try_from(count).map_err(|_| {
                    runtime(
                        RuntimeErrorKind::Arithmetic(
                            "count exceeds the signed 64-bit range".to_owned(),
                        ),
                        instruction.span,
                    )
                })?))
            }
            Builtin::VarDump => {
                for argument in arguments {
                    self.write_output(&argument.dump(), instruction.span)?;
                }
                Ok(Value::NULL)
            }
            Builtin::MemoryStreamOpen => {
                let bytes = arguments
                    .first()
                    .map_or_else(Vec::new, |value| value.as_bytes().unwrap().to_vec());
                Value::try_stream(self.result_class(instruction)?, bytes)
                    .map_err(|kind| runtime(kind, instruction.span))
            }
            Builtin::TempStreamOpen => {
                let threshold = if let Some(threshold) = arguments.first().and_then(Value::as_int) {
                    self.nonnegative_stream_value(
                        threshold,
                        "temporary stream threshold",
                        instruction.span,
                    )?
                } else {
                    2 * 1024 * 1024
                };
                Value::try_temp_stream(self.result_class(instruction)?, threshold)
                    .map_err(|kind| runtime(kind, instruction.span))
            }
            Builtin::StreamsOpen => {
                let uri = arguments[0].as_bytes().expect("verified URI string");
                if !(uri == b"thp:/input"
                    || uri == b"php://memory"
                    || uri.starts_with(b"php://temp"))
                {
                    return Err(self.native_exception(
                        "InvalidStreamUriException",
                        b"invalid stream URI".to_vec(),
                        Some(uri.to_vec()),
                        0,
                        instruction.span,
                    ));
                }
                if uri == b"thp:/input" {
                    if arguments[1].as_int() != Some(0) {
                        return Err(self.native_exception(
                            "UnsupportedStreamOperationException",
                            b"thp:/input is read-only".to_vec(),
                            Some(uri.to_vec()),
                            0,
                            instruction.span,
                        ));
                    }
                    if let Some(stream) = &self.request_input_stream {
                        return Ok(stream.clone());
                    }
                    let stream = Value::try_request_input_stream(
                        self.class_named("ReadableFileStream", instruction.span)?,
                        self.request_input.clone(),
                    )
                    .map_err(|kind| runtime(kind, instruction.span))?;
                    self.request_input_stream = Some(stream.clone());
                    Ok(stream)
                } else if uri.starts_with(b"php://temp") {
                    let threshold = uri
                        .strip_prefix(b"php://temp/maxmemory:")
                        .map(|threshold| {
                            std::str::from_utf8(threshold)
                                .ok()
                                .and_then(|threshold| threshold.parse::<usize>().ok())
                                .ok_or_else(|| {
                                    self.native_exception(
                                        "InvalidStreamUriException",
                                        b"invalid temporary-stream threshold".to_vec(),
                                        Some(uri.to_vec()),
                                        0,
                                        instruction.span,
                                    )
                                })
                        })
                        .transpose()?
                        .unwrap_or(2 * 1024 * 1024);
                    Value::try_temp_stream(self.result_class(instruction)?, threshold)
                        .map_err(|kind| runtime(kind, instruction.span))
                } else {
                    Value::try_stream(self.result_class(instruction)?, Vec::new())
                        .map_err(|kind| runtime(kind, instruction.span))
                }
            }
            Builtin::FilesOpenRead => {
                let path =
                    std::str::from_utf8(arguments[0].as_bytes().expect("verified path string"))
                        .map_err(|_| {
                            runtime(
                                RuntimeErrorKind::Io("file path must be UTF-8".to_owned()),
                                instruction.span,
                            )
                        })?;
                let resolved = if Path::new(path).is_relative() {
                    self.filesystem_base.join(path)
                } else {
                    PathBuf::from(path)
                };
                let bytes = std::fs::read(resolved).map_err(|error| {
                    self.native_exception(
                        "OpenStreamException",
                        format!("cannot open `{path}`: {error}").into_bytes(),
                        Some(path.as_bytes().to_vec()),
                        i64::from(error.raw_os_error().unwrap_or(-1)),
                        instruction.span,
                    )
                })?;
                Value::try_stream(self.result_class(instruction)?, bytes)
                    .map_err(|kind| runtime(kind, instruction.span))
            }
            Builtin::StreamTell => {
                let position = arguments[0]
                    .stream_tell()
                    .map_err(|kind| self.stream_exception(kind, instruction.span))?;
                Ok(Value::integer(i64::try_from(position).map_err(|_| {
                    runtime(
                        RuntimeErrorKind::Arithmetic(
                            "stream position exceeds the signed 64-bit range".to_owned(),
                        ),
                        instruction.span,
                    )
                })?))
            }
            Builtin::StreamRead => {
                let length = self.nonnegative_stream_value(
                    arguments[1].as_int().expect("verified read length"),
                    "stream read length",
                    instruction.span,
                )?;
                Value::try_bytes(
                    arguments[0]
                        .stream_read(length)
                        .map_err(|kind| self.stream_exception(kind, instruction.span))?,
                )
                .map_err(|kind| runtime(kind, instruction.span))
            }
            Builtin::StreamReadAll => {
                let limit = arguments
                    .get(1)
                    .filter(|value| !value.is_null())
                    .map(|value| {
                        self.nonnegative_stream_value(
                            value.as_int().expect("verified read limit"),
                            "stream read limit",
                            instruction.span,
                        )
                    })
                    .transpose()?;
                Value::try_bytes(
                    arguments[0]
                        .stream_read_all(limit)
                        .map_err(|kind| self.stream_exception(kind, instruction.span))?,
                )
                .map_err(|kind| runtime(kind, instruction.span))
            }
            Builtin::StreamEof => {
                Ok(Value::bool(arguments[0].stream_eof().map_err(|kind| {
                    self.stream_exception(kind, instruction.span)
                })?))
            }
            Builtin::StreamSeek => {
                let position = self.nonnegative_stream_value(
                    arguments[1].as_int().expect("verified seek position"),
                    "stream seek position",
                    instruction.span,
                )?;
                arguments[0]
                    .stream_seek(position)
                    .map_err(|kind| self.stream_exception(kind, instruction.span))?;
                Ok(Value::NULL)
            }
            Builtin::StreamWriteAll => {
                arguments[0]
                    .stream_write_all(arguments[1].as_bytes().expect("verified write bytes"))
                    .map_err(|kind| self.stream_exception(kind, instruction.span))?;
                Ok(Value::NULL)
            }
            Builtin::StreamClose => {
                arguments[0]
                    .stream_close()
                    .map_err(|kind| self.stream_exception(kind, instruction.span))?;
                Ok(Value::NULL)
            }
            Builtin::StreamIsClosed => {
                Ok(Value::bool(arguments[0].stream_is_closed().map_err(
                    |kind| self.stream_exception(kind, instruction.span),
                )?))
            }
            Builtin::ExceptionNew => {
                let message = arguments
                    .first()
                    .map_or_else(Vec::new, |value| value.as_bytes().unwrap().to_vec());
                Value::try_exception(self.result_class(instruction)?, message, None, 0)
                    .map_err(|kind| runtime(kind, instruction.span))
            }
            Builtin::ExceptionConstruct => {
                let previous = (!arguments[3].is_null()).then(|| arguments[3].clone());
                arguments[0]
                    .initialize_exception(
                        arguments[1]
                            .as_bytes()
                            .expect("verified exception message")
                            .to_vec(),
                        arguments[2].as_int().expect("verified exception code"),
                        previous,
                    )
                    .map_err(|kind| runtime(kind, instruction.span))?;
                Ok(Value::NULL)
            }
            Builtin::ExceptionGetMessage => Value::try_bytes(
                arguments[0]
                    .exception_message()
                    .map_err(|kind| runtime(kind, instruction.span))?,
            )
            .map_err(|kind| runtime(kind, instruction.span)),
            Builtin::ExceptionGetCode => Ok(Value::integer(
                arguments[0]
                    .exception_code()
                    .map_err(|kind| runtime(kind, instruction.span))?,
            )),
            Builtin::ExceptionGetPrevious => arguments[0]
                .exception_previous()
                .map_err(|kind| runtime(kind, instruction.span)),
            Builtin::ExceptionGetTarget => Value::try_bytes(
                arguments[0]
                    .exception_target()
                    .map_err(|kind| runtime(kind, instruction.span))?
                    .to_vec(),
            )
            .map_err(|kind| runtime(kind, instruction.span)),
            Builtin::ExceptionGetSystemCode => Ok(Value::integer(
                arguments[0]
                    .exception_system_code()
                    .map_err(|kind| runtime(kind, instruction.span))?,
            )),
            Builtin::ExceptionGetSuppressed => Value::try_vector(
                Type::Object("Throwable".to_owned()),
                arguments[0]
                    .exception_suppressed()
                    .map_err(|kind| runtime(kind, instruction.span))?,
            )
            .map_err(|kind| runtime(kind, instruction.span)),
        }
    }

    fn result_class(&self, instruction: &Instruction) -> Result<thp_hir::ClassId, VmError> {
        let Some(Type::Object(name)) = instruction.ty.as_ref() else {
            return Err(runtime(RuntimeErrorKind::Unreachable, instruction.span));
        };
        self.program
            .classes
            .iter()
            .find(|class| &class.name == name)
            .map(|class| class.id)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, instruction.span))
    }

    fn class_named(&self, name: &str, span: Span) -> Result<thp_hir::ClassId, VmError> {
        self.program
            .classes
            .iter()
            .find(|class| class.name == name)
            .map(|class| class.id)
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))
    }

    fn native_exception(
        &self,
        class_name: &str,
        message: Vec<u8>,
        target: Option<Vec<u8>>,
        system_code: i64,
        span: Span,
    ) -> VmError {
        let class = self
            .program
            .classes
            .iter()
            .find(|class| class.name == class_name)
            .expect("native exception class is present")
            .id;
        match Value::try_exception(class, message, target, system_code) {
            Ok(value) => VmError::Thrown {
                value,
                span,
                trace: Vec::new(),
            },
            Err(kind) => runtime(kind, span),
        }
    }

    fn stream_exception(&self, kind: RuntimeErrorKind, span: Span) -> VmError {
        if matches!(
            kind,
            RuntimeErrorKind::HeapLimit { .. }
                | RuntimeErrorKind::AllocationFailure
                | RuntimeErrorKind::InputSizeLimit { .. }
                | RuntimeErrorKind::InputTimeLimit { .. }
                | RuntimeErrorKind::StackDepthLimit { .. }
                | RuntimeErrorKind::OpenHandleLimit { .. }
                | RuntimeErrorKind::OutputIo(_)
                | RuntimeErrorKind::CrossRequestValue
        ) {
            return runtime(kind, span);
        }
        let (class, message) = match kind {
            RuntimeErrorKind::Io(message) if message == "stream is closed" => {
                ("ClosedStreamException", message.into_bytes())
            }
            other => ("IoException", other.to_string().into_bytes()),
        };
        self.native_exception(class, message, None, 0, span)
    }

    fn nonnegative_stream_value(
        &self,
        value: i64,
        name: &str,
        span: Span,
    ) -> Result<usize, VmError> {
        usize::try_from(value).map_err(|_| {
            self.native_exception(
                "ValueError",
                format!("{name} cannot be negative").into_bytes(),
                None,
                0,
                span,
            )
        })
    }

    fn tick(&mut self, span: Span) -> Result<(), VmError> {
        self.request_heap.collect_if_needed();
        self.instructions += 1;
        if self
            .limits
            .max_instructions
            .is_some_and(|limit| self.instructions > limit)
        {
            return Err(VmError::InstructionLimit {
                limit: self.limits.max_instructions.expect("limit exists"),
                span,
            });
        }
        if self
            .limits
            .max_execution
            .is_some_and(|limit| self.started.elapsed() >= limit)
        {
            return Err(VmError::ExecutionTimeLimit {
                limit: self.limits.max_execution.expect("limit exists"),
                span,
            });
        }
        Ok(())
    }

    fn write_output(&mut self, bytes: &[u8], span: Span) -> Result<(), VmError> {
        self.output
            .write_all(bytes)
            .map_err(|error| VmError::OutputIo {
                message: error.to_string(),
                span,
            })?;
        self.output_bytes = self
            .output_bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| VmError::OutputIo {
                message: "output byte counter overflow".to_owned(),
                span,
            })?;
        Ok(())
    }
}

fn catch_exception(
    program: &Program,
    function: &Function,
    frame: &mut Frame,
    throwing_block: BlockId,
    value: Value,
) -> bool {
    let Some(class) = value.class_id() else {
        return false;
    };
    let Some(clause) = function
        .exception_handlers
        .iter()
        .filter(|handler| handler.protected_blocks.contains(&throwing_block))
        .find_map(|handler| {
            handler.catches.iter().find(|clause| {
                clause
                    .class
                    .is_none_or(|caught| is_instance_of(program, class, caught))
            })
        })
    else {
        return false;
    };
    frame.locals[clause.local.0 as usize] = Some(value);
    frame.previous = Some(frame.current);
    frame.current = clause.target;
    true
}

fn is_instance_of_name(program: &Program, actual: ClassId, expected: &str) -> bool {
    program
        .classes
        .iter()
        .find(|class| class.name == expected)
        .is_some_and(|expected| is_instance_of(program, actual, expected.id))
}

fn is_instance_of(
    program: &Program,
    mut actual: thp_hir::ClassId,
    expected: thp_hir::ClassId,
) -> bool {
    loop {
        if actual == expected
            || program.classes[actual.0 as usize]
                .interfaces
                .contains(&expected)
        {
            return true;
        }
        let Some(parent) = program.classes[actual.0 as usize].parent else {
            return false;
        };
        actual = parent;
    }
}

fn get_register(frame: &Frame, register: Register, span: Span) -> Result<Value, VmError> {
    frame.registers[register.0 as usize]
        .clone()
        .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))
}

fn collection_offset(frame: &Frame, register: Register, span: Span) -> Result<usize, VmError> {
    let value = get_register(frame, register, span)?;
    let offset = value
        .as_int()
        .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
    usize::try_from(offset).map_err(|_| {
        runtime(
            RuntimeErrorKind::Bounds("negative collection iteration offset".to_owned()),
            span,
        )
    })
}

fn describe_match_subject(value: &Value) -> String {
    if value.is_null() {
        return "null".to_owned();
    }
    if let Some(value) = value.as_int() {
        return format!("int {value}");
    }
    if let Some(value) = value.as_float() {
        return format!("float {value}");
    }
    if let Some(value) = value.as_bool() {
        return format!("bool {value}");
    }
    if let Some(bytes) = value.as_bytes() {
        let escaped = bytes
            .iter()
            .take(64)
            .flat_map(|byte| std::ascii::escape_default(*byte))
            .map(char::from)
            .collect::<String>();
        let suffix = if bytes.len() > 64 { "…" } else { "" };
        return format!("string \"{escaped}{suffix}\"");
    }
    format!("value of type {}", value.type_name())
}

fn constant_value(constant: &Constant, span: Span) -> Result<Value, VmError> {
    Ok(match constant {
        Constant::Integer(value) => Value::integer(*value),
        Constant::Float(value) => Value::float(*value),
        Constant::Bool(value) => Value::bool(*value),
        Constant::Null => Value::NULL,
        Constant::String(value) => {
            Value::try_bytes(value.clone()).map_err(|kind| runtime(kind, span))?
        }
    })
}

fn execute_unary(op: UnaryOp, operand: &Value, span: Span) -> Result<Value, VmError> {
    match op {
        UnaryOp::Negate => {
            if let Some(value) = operand.as_int() {
                value.checked_neg().map(Value::integer).ok_or_else(|| {
                    runtime(
                        RuntimeErrorKind::Arithmetic("integer negation overflow".to_owned()),
                        span,
                    )
                })
            } else if let Some(value) = operand.as_float() {
                Ok(Value::float(-value))
            } else {
                Err(runtime(
                    RuntimeErrorKind::TypeError("unary `-` requires a number".to_owned()),
                    span,
                ))
            }
        }
        UnaryOp::Not => operand
            .as_bool()
            .map(|value| Value::bool(!value))
            .ok_or_else(|| {
                runtime(
                    RuntimeErrorKind::TypeError("unary `!` requires bool".to_owned()),
                    span,
                )
            }),
    }
}

#[allow(clippy::too_many_lines)]
fn execute_binary(op: BinaryOp, left: &Value, right: &Value, span: Span) -> Result<Value, VmError> {
    if op == BinaryOp::Concatenate {
        let mut bytes = left
            .output_bytes()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        let right = right
            .output_bytes()
            .ok_or_else(|| runtime(RuntimeErrorKind::Unreachable, span))?;
        bytes
            .try_reserve(right.len())
            .map_err(|_| runtime(RuntimeErrorKind::AllocationFailure, span))?;
        bytes.extend_from_slice(&right);
        return Value::try_bytes(bytes).map_err(|kind| runtime(kind, span));
    }

    if let (Some(left), Some(right)) = (left.as_int(), right.as_int()) {
        return match op {
            BinaryOp::Add => checked_int(left.checked_add(right), "addition", span),
            BinaryOp::Subtract => checked_int(left.checked_sub(right), "subtraction", span),
            BinaryOp::Multiply => checked_int(left.checked_mul(right), "multiplication", span),
            BinaryOp::Divide => {
                if right == 0 {
                    Err(runtime(
                        RuntimeErrorKind::Arithmetic("division by zero".to_owned()),
                        span,
                    ))
                } else {
                    checked_int(left.checked_div(right), "division", span)
                }
            }
            BinaryOp::Remainder => {
                if right == 0 {
                    Err(runtime(
                        RuntimeErrorKind::Arithmetic("remainder by zero".to_owned()),
                        span,
                    ))
                } else {
                    checked_int(left.checked_rem(right), "remainder", span)
                }
            }
            BinaryOp::Equal | BinaryOp::StrictEqual => Ok(Value::bool(left == right)),
            BinaryOp::NotEqual => Ok(Value::bool(left != right)),
            BinaryOp::Less => Ok(Value::bool(left < right)),
            BinaryOp::LessEqual => Ok(Value::bool(left <= right)),
            BinaryOp::Greater => Ok(Value::bool(left > right)),
            BinaryOp::GreaterEqual => Ok(Value::bool(left >= right)),
            BinaryOp::Concatenate | BinaryOp::And | BinaryOp::Or | BinaryOp::Coalesce => {
                Err(runtime(RuntimeErrorKind::Unreachable, span))
            }
        };
    }
    if let (Some(left), Some(right)) = (left.as_float(), right.as_float()) {
        return Ok(match op {
            BinaryOp::Add => Value::float(left + right),
            BinaryOp::Subtract => Value::float(left - right),
            BinaryOp::Multiply => Value::float(left * right),
            BinaryOp::Divide => Value::float(left / right),
            BinaryOp::Remainder => Value::float(left % right),
            BinaryOp::Equal | BinaryOp::StrictEqual => Value::bool(left == right),
            BinaryOp::NotEqual => Value::bool(left != right),
            BinaryOp::Less => Value::bool(left < right),
            BinaryOp::LessEqual => Value::bool(left <= right),
            BinaryOp::Greater => Value::bool(left > right),
            BinaryOp::GreaterEqual => Value::bool(left >= right),
            BinaryOp::Concatenate | BinaryOp::And | BinaryOp::Or | BinaryOp::Coalesce => {
                return Err(runtime(RuntimeErrorKind::Unreachable, span));
            }
        });
    }
    if let (Some(left), Some(right)) = (left.as_bool(), right.as_bool()) {
        return Ok(match op {
            BinaryOp::Equal | BinaryOp::StrictEqual => Value::bool(left == right),
            BinaryOp::NotEqual => Value::bool(left != right),
            BinaryOp::And => Value::bool(left && right),
            BinaryOp::Or => Value::bool(left || right),
            _ => return Err(runtime(RuntimeErrorKind::Unreachable, span)),
        });
    }
    if let (Some(left), Some(right)) = (left.as_bytes(), right.as_bytes()) {
        return Ok(match op {
            BinaryOp::Equal | BinaryOp::StrictEqual => Value::bool(left == right),
            BinaryOp::NotEqual => Value::bool(left != right),
            BinaryOp::Less => Value::bool(left < right),
            BinaryOp::LessEqual => Value::bool(left <= right),
            BinaryOp::Greater => Value::bool(left > right),
            BinaryOp::GreaterEqual => Value::bool(left >= right),
            _ => return Err(runtime(RuntimeErrorKind::Unreachable, span)),
        });
    }
    if matches!(
        op,
        BinaryOp::Equal | BinaryOp::StrictEqual | BinaryOp::NotEqual
    ) {
        return Ok(Value::bool(if op == BinaryOp::NotEqual {
            left != right
        } else {
            left == right
        }));
    }
    Err(runtime(
        RuntimeErrorKind::TypeError(format!(
            "operator cannot be applied to {} and {}",
            left.type_name(),
            right.type_name()
        )),
        span,
    ))
}

fn checked_int(value: Option<i64>, operation: &str, span: Span) -> Result<Value, VmError> {
    value.map(Value::integer).ok_or_else(|| {
        runtime(
            RuntimeErrorKind::Arithmetic(format!("integer {operation} overflow")),
            span,
        )
    })
}

fn runtime(kind: RuntimeErrorKind, span: Span) -> VmError {
    VmError::Runtime(RuntimeError::new(kind, span))
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::path::PathBuf;
    use std::time::Duration;

    use thp_bytecode::lower as lower_bytecode;
    use thp_diagnostics::SourceFile;
    use thp_hir::lower as lower_hir;
    use thp_mir::lower as lower_mir;
    use thp_runtime::{RequestInput, RuntimeError, RuntimeErrorKind};
    use thp_syntax::parse;

    use super::{ExecutionContext, Limits, VmError, execute, execute_captured, execute_to};

    fn run(source: &str) -> Result<super::Execution, VmError> {
        let source = SourceFile::new("test.thp", source);
        let parsed = parse(&source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let hir = lower_hir(&parsed.program);
        assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);
        let mir = lower_mir(&hir.module);
        let bytecode = lower_bytecode(&mir);
        execute(&bytecode, Limits::default())
    }

    #[test]
    fn executes_functions_loops_and_output() {
        let execution = run(r#"<?thp
function double(int $value): int { return $value * 2; }
$index: int = 0;
while ($index < 3) {
    echo double($index) . "\n";
    $index = $index + 1;
}
"#)
        .unwrap();
        assert_eq!(execution.output, b"0\n2\n4\n");
        assert!(execution.instructions > 0);
    }

    #[test]
    fn output_scalars_and_concatenation_share_canonical_formatting() {
        let execution = run(r#"<?thp
echo "text\n";
echo 42;
echo "\n";
echo 1.0;
echo "\n";
echo -0.0;
echo "\n";
echo 0.1 + 0.2;
echo "\n";
echo 1.0 / 0.0;
echo "\n";
echo -1.0 / 0.0;
echo "\n";
echo 0.0 / 0.0;
echo "\n";
echo true;
echo "\n";
echo false;
echo "\n";
echo "[" . false . "]";
"#)
        .unwrap();
        assert_eq!(
            execution.output,
            b"text\n42\n1.0\n-0.0\n0.30000000000000004\nINF\n-INF\nNAN\ntrue\nfalse\n[false]"
        );
    }

    #[test]
    fn preserves_short_circuit_behavior() {
        let execution =
            run("<?thp\nif (false && (1 / 0 === 0)) { echo \"bad\"; } echo \"ok\";").unwrap();
        assert_eq!(execution.output, b"ok");
    }

    #[test]
    fn uncaught_match_error_retains_class_message_and_call_trace() {
        let error = run(r#"<?thp
function inner(): string {
    return match (9) { 1 => "one" };
}
function outer(): string {
    return inner();
}
echo outer();
"#)
        .unwrap_err();
        let VmError::Runtime(error) = error else {
            panic!("expected an uncaught runtime error");
        };
        assert!(matches!(
            error.kind,
            thp_runtime::RuntimeErrorKind::UncaughtException {
                ref class,
                ref message,
            } if class == "UnhandledMatchError"
                && message == "no match arm handled int 9"
        ));
        assert_eq!(error.trace.len(), 2);
        assert_eq!(error.trace[0].function, "outer");
        assert_eq!(error.trace[1].function, "<main>");
    }

    #[test]
    fn executes_object_construction_properties_and_methods() {
        let execution = run(r#"<?thp
class Counter {
    private int $value;

    public function __construct(int $initial) {
        $this->value = $initial;
    }

    public function increment(): int {
        $this->value = $this->value + 1;
        return $this->value;
    }
}

$counter = new Counter(40);
echo $counter->increment() . "\n";
echo $counter->increment() . "\n";
var_dump($counter instanceof Counter);
"#)
        .unwrap();
        assert_eq!(execution.output, b"41\n42\nbool(true)\n");
    }

    #[test]
    fn catches_user_objects_thrown_across_a_call_frame() {
        let execution = run(r#"<?thp
class Problem extends Exception {}
function fail(): void {
    throw new Problem("caught");
}
try {
    fail();
    echo "unreachable";
} catch (Problem $error) {
    echo $error->getMessage() . "\n";
}
"#)
        .unwrap();
        assert_eq!(execution.output, b"caught\n");
    }

    #[test]
    fn using_closes_on_fallthrough_return_and_exception() {
        let execution = run(r#"<?thp
class Problem extends Exception {}
class Probe implements Closeable {
    private bool $closed = false;
    private string $name;
    public function __construct(string $name) {
        $this->name = $name;
    }
    public function close(): void {
        if (!$this->closed) {
            $this->closed = true;
            echo "close:" . $this->name . "\n";
        }
    }
    public function isClosed(): bool {
        return $this->closed;
    }
}
function returnFromUsing(): string {
    using ($probe = new Probe("return")) {
        return "returned";
    }
}
using ($probe = new Probe("fallthrough")) {
    echo "body\n";
}
echo returnFromUsing() . "\n";
try {
    using ($probe = new Probe("exception")) {
        throw new Problem();
    }
} catch (Problem $error) {
    echo "caught\n";
}
"#)
        .unwrap();
        assert_eq!(
            execution.output,
            b"body\nclose:fallthrough\nclose:return\nreturned\nclose:exception\ncaught\n"
        );
    }

    #[test]
    fn memory_streams_preserve_bytes_and_share_cursor_state() {
        let execution = run(r#"<?thp
$bytes = "\x00\xffTHP";
$stream = MemoryStream::open($bytes);
$alias = $stream;
var_dump($stream->tell());
var_dump($stream->readAll() === $bytes);
var_dump($alias->eof());

$stream = MemoryStream::open("abcd");
$alias = $stream;
echo $stream->read(2) . "\n";
echo $alias->read(2) . "\n";
$alias->seek(6);
$alias->writeAll("z");
$stream->seek(0);
var_dump($stream->readAll() === "abcd\x00\x00z");
"#)
        .unwrap();
        assert_eq!(
            execution.output,
            b"int(0)\nbool(true)\nbool(true)\nab\ncd\nbool(true)\n"
        );
    }

    #[test]
    fn stream_uri_and_temp_factories_execute() {
        let execution = run(r#"<?thp
$memory = Streams::open("php://memory", OpenMode::ReadWrite);
if (
    $memory instanceof ReadableStream
    && $memory instanceof WritableStream
    && $memory instanceof SeekableStream
) {
    $memory->writeAll("uri");
    $memory->seek(0);
    echo $memory->readAll() . "\n";
}

$temp = TempStream::open(0);
$temp->writeAll("disk-backed");
$temp->seek(0);
echo $temp->readAll() . "\n";
"#)
        .unwrap();
        assert_eq!(execution.output, b"uri\ndisk-backed\n");
    }

    #[test]
    fn using_closes_native_stream_aliases() {
        let execution = run(r#"<?thp
using ($stream = MemoryStream::open()) {
    $alias = $stream;
    $stream->writeAll("payload");
}
var_dump($alias->isClosed());
"#)
        .unwrap();
        assert_eq!(execution.output, b"bool(true)\n");
    }

    #[test]
    fn native_stream_failures_are_typed_and_catchable() {
        let execution = run(r#"<?thp
try {
    Streams::open("unknown://target", OpenMode::Read);
} catch (InvalidStreamUriException $error) {
    echo "invalid uri\n";
}
$stream = MemoryStream::open("12345");
try {
    $stream->read(-1);
} catch (ValueError $error) {
    echo "invalid length\n";
}
try {
    $stream->readAll(4);
} catch (IoException $error) {
    echo "limit exceeded\n";
}
var_dump($stream->tell());
$stream->close();
try {
    $stream->seek(0);
} catch (IoException $error) {
    echo "closed stream\n";
}
"#)
        .unwrap();
        assert_eq!(
            execution.output,
            b"invalid uri\ninvalid length\nlimit exceeded\nint(0)\nclosed stream\n"
        );
    }

    #[test]
    fn native_exception_methods_expose_message_and_file_context() {
        let execution = run(r#"<?thp
try {
    throw new Exception("body failed");
} catch (Exception $error) {
    echo $error->getMessage() . "\n";
}
try {
    Files::openRead("./definitely-missing-thp-resource-test");
} catch (OpenStreamException $error) {
    echo $error->getTarget() . "\n";
    var_dump($error->getSystemCode() !== 0);
}
"#)
        .unwrap();
        assert_eq!(
            execution.output,
            b"body failed\n./definitely-missing-thp-resource-test\nbool(true)\n"
        );
    }

    #[test]
    fn using_preserves_body_exception_and_suppresses_cleanup_failure() {
        let execution = run(r#"<?thp
class FailingClose implements Closeable {
    private bool $closed = false;
    public function close(): void {
        if (!$this->closed) {
            $this->closed = true;
            throw new IoException("close failed");
        }
    }
    public function isClosed(): bool {
        return $this->closed;
    }
}
try {
    using ($handle = new FailingClose()) {
        throw new Exception("body failed");
    }
} catch (Exception $error) {
    echo $error->getMessage() . "\n";
    var_dump(count($error->getSuppressed()));
    echo $error->getSuppressed()[0]->getMessage() . "\n";
}
"#)
        .unwrap();
        assert_eq!(execution.output, b"body failed\nint(1)\nclose failed\n");
    }

    #[test]
    fn traps_integer_overflow() {
        let error = run("<?thp\n$value = 9223372036854775807 + 1;").unwrap_err();
        assert!(matches!(error, VmError::Runtime(_)));
    }

    #[test]
    fn enforces_instruction_limit() {
        let source = SourceFile::new("test.thp", "<?thp\nwhile (true) {}");
        let parsed = parse(&source);
        let hir = lower_hir(&parsed.program);
        let bytecode = lower_bytecode(&lower_mir(&hir.module));
        let error = execute(
            &bytecode,
            Limits {
                max_instructions: Some(10),
                max_execution: None,
                ..Limits::default()
            },
        )
        .unwrap_err();
        assert!(matches!(error, VmError::InstructionLimit { .. }));
    }

    #[test]
    fn captured_execution_retains_partial_output_and_metrics() {
        let source = SourceFile::new(
            "test.thp",
            "<?thp\necho \"before\\n\";\n$value = 9223372036854775807 + 1;",
        );
        let parsed = parse(&source);
        let hir = lower_hir(&parsed.program);
        let bytecode = lower_bytecode(&lower_mir(&hir.module));
        let failure = execute_captured(&bytecode, &ExecutionContext::default())
            .expect_err("integer overflow");
        assert_eq!(failure.output, b"before\n");
        assert!(failure.instructions > 0);
        assert!(matches!(failure.error, VmError::Runtime(_)));
    }

    #[test]
    fn execution_timeout_is_checked_cooperatively() {
        let source = SourceFile::new("test.thp", "<?thp\nwhile (true) {}");
        let parsed = parse(&source);
        let hir = lower_hir(&parsed.program);
        let bytecode = lower_bytecode(&lower_mir(&hir.module));
        let failure = execute_captured(
            &bytecode,
            &ExecutionContext {
                limits: Limits {
                    max_instructions: None,
                    max_execution: Some(Duration::ZERO),
                    ..Limits::default()
                },
                filesystem_base: PathBuf::new(),
                request_input: RequestInput::empty(),
            },
        )
        .expect_err("time limit");
        assert!(matches!(failure.error, VmError::ExecutionTimeLimit { .. }));
    }

    fn bytecode(source: &str) -> thp_bytecode::Program {
        let source = SourceFile::new("test.thp", source);
        let parsed = parse(&source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let hir = lower_hir(&parsed.program);
        assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);
        lower_bytecode(&lower_mir(&hir.module))
    }

    #[test]
    fn thp_input_streams_binary_body_through_one_cursor() {
        let program = bytecode(
            r#"<?thp
$first = Streams::open("thp:/input", OpenMode::Read);
$second = Streams::open("thp:/input", OpenMode::Read);
echo $first->read(2);
echo $second->readAll();
"#,
        );
        let input =
            RequestInput::from_bytes(b"ab\0cd".to_vec(), Some(5), None).expect("request input");
        let execution = execute_captured(
            &program,
            &ExecutionContext {
                limits: Limits {
                    max_input_bytes: Some(5),
                    ..Limits::default()
                },
                filesystem_base: PathBuf::new(),
                request_input: input,
            },
        )
        .unwrap();
        assert_eq!(execution.output, b"ab\0cd");
        assert_eq!(execution.output_bytes, 5);
    }

    #[test]
    fn engine_input_limit_preflights_host_declared_length() {
        let program = bytecode("<?thp\necho \"unreached\";");
        let input =
            RequestInput::from_bytes(b"oversized".to_vec(), None, None).expect("request input");
        let failure = execute_captured(
            &program,
            &ExecutionContext {
                limits: Limits {
                    max_input_bytes: Some(4),
                    ..Limits::default()
                },
                filesystem_base: PathBuf::new(),
                request_input: input,
            },
        )
        .expect_err("engine limit rejects the declared body before execution");
        assert!(matches!(
            failure.error,
            VmError::Runtime(RuntimeError {
                kind: RuntimeErrorKind::InputSizeLimit { limit: 4 },
                ..
            })
        ));
        assert_eq!(failure.instructions, 0);
        assert_eq!(failure.output_bytes, 0);
    }

    #[test]
    fn enforces_heap_stack_and_open_handle_limits_as_request_failures() {
        let heap_program = bytecode(&format!("<?thp\n$value = \"{}\";", "x".repeat(4096)));
        let heap_failure = execute_captured(
            &heap_program,
            &ExecutionContext {
                limits: Limits {
                    max_heap_bytes: Some(1024),
                    ..Limits::default()
                },
                ..ExecutionContext::default()
            },
        )
        .unwrap_err();
        assert!(matches!(
            heap_failure.error,
            VmError::Runtime(RuntimeError {
                kind: RuntimeErrorKind::HeapLimit { .. },
                ..
            })
        ));

        let stack_program = bytecode(
            r"<?thp
function recurse(int $depth): int {
    if ($depth === 0) { return 0; }
    return recurse($depth - 1);
}
$result = recurse(8);
",
        );
        let stack_failure = execute_captured(
            &stack_program,
            &ExecutionContext {
                limits: Limits {
                    max_stack_depth: Some(4),
                    ..Limits::default()
                },
                ..ExecutionContext::default()
            },
        )
        .unwrap_err();
        assert!(matches!(
            stack_failure.error,
            VmError::StackDepthLimit { limit: 4, .. }
        ));

        let handle_program = bytecode(
            r"<?thp
$first = MemoryStream::open();
$second = MemoryStream::open();
",
        );
        let handle_failure = execute_captured(
            &handle_program,
            &ExecutionContext {
                limits: Limits {
                    max_open_handles: Some(1),
                    ..Limits::default()
                },
                ..ExecutionContext::default()
            },
        )
        .unwrap_err();
        assert!(matches!(
            handle_failure.error,
            VmError::Runtime(RuntimeError {
                kind: RuntimeErrorKind::OpenHandleLimit { limit: 1 },
                ..
            })
        ));
        assert_eq!(handle_failure.heap.open_handles, 0);
        assert_eq!(handle_failure.heap.live_cells, 0);
    }

    struct RejectSecondWrite {
        writes: usize,
        bytes: Vec<u8>,
    }

    impl Write for RejectSecondWrite {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.writes += 1;
            if self.writes == 2 {
                return Err(io::Error::other("sink closed"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn streaming_output_preserves_backpressure_and_does_not_capture() {
        let program = bytecode("<?thp\necho \"first\";\necho \"second\";");
        let mut sink = RejectSecondWrite {
            writes: 0,
            bytes: Vec::new(),
        };
        let failure = execute_to(&program, &ExecutionContext::default(), &mut sink)
            .expect_err("second sink write fails");
        assert!(matches!(failure.error, VmError::OutputIo { .. }));
        assert_eq!(failure.output_bytes, 5);
        assert_eq!(sink.bytes, b"first");
    }
}
