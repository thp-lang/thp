//! The reference interpreter for verified THP bytecode.

#![allow(clippy::float_cmp, clippy::too_many_lines)]

use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use thp_bytecode::{
    Function, Instruction, InstructionKind, Method, Program, Property, Terminator,
    VerificationError, verify,
};
use thp_diagnostics::Span;
use thp_hir::{
    Builtin, CalledClass, Callee, ClassId, ConstantValue, FunctionId, MethodSlot,
    ParameterMetadata, ReflectionBuiltin, Type,
};
use thp_mir::{BlockId, Constant, Register};
use thp_runtime::{
    HeapStats, ReflectionCallable, ReflectionValue, RequestHeap, RequestInput, RuntimeError,
    RuntimeErrorKind, StackFrame, Value,
};
use thp_syntax::{BinaryOp, UnaryOp, Visibility};

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
        classes_by_name: program
            .classes
            .iter()
            .map(|class| (class.name.as_str(), class.id))
            .collect(),
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
    classes_by_name: HashMap<&'program str, ClassId>,
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
    #[allow(clippy::too_many_arguments)]
    fn dynamic_new(
        &mut self,
        target: &Value,
        supplied_types: &[Type],
        arguments: Vec<(Option<String>, Value, Type, Span)>,
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        let bytes = target.as_bytes().ok_or_else(|| {
            runtime(
                RuntimeErrorKind::TypeError("dynamic class target is not string".to_owned()),
                instruction.span,
            )
        })?;
        let name = std::str::from_utf8(bytes).map_err(|_| {
            self.native_exception(
                "ValueError",
                b"dynamic class name is not valid UTF-8".to_vec(),
                None,
                0,
                instruction.span,
            )
        })?;
        let class_id = self.classes_by_name.get(name).copied().ok_or_else(|| {
            self.native_exception(
                "Error",
                format!("unknown dynamic class `{name}`").into_bytes(),
                None,
                0,
                instruction.span,
            )
        })?;
        let class = self.program.classes[class_id.0 as usize].clone();
        let fail = |message: String| {
            self.native_exception("Error", message.into_bytes(), None, 0, instruction.span)
        };
        if class.kind != thp_hir::NominalKind::Class || class.abstract_class || class.native {
            return Err(fail(format!(
                "class `{name}` is not dynamically constructible"
            )));
        }
        if supplied_types.len() > class.type_parameters.len() {
            return Err(fail(format!(
                "class `{name}` expects at most {} generic arguments, found {}",
                class.type_parameters.len(),
                supplied_types.len()
            )));
        }
        let mut type_arguments = supplied_types.to_vec();
        type_arguments.resize(class.type_parameters.len(), Type::Mixed);
        let instance = thp_bytecode::NominalType {
            class: class.id,
            arguments: type_arguments.clone(),
        };
        for (parameter, argument) in class.type_parameters.iter().zip(&type_arguments) {
            if let Some(bound) = &parameter.bound {
                let bound = substitute_runtime_type(bound, Some(&instance), &class);
                if argument == &Type::Mixed || !descriptor_accepts(self.program, &bound, argument) {
                    return Err(fail(format!(
                        "generic argument `{}` does not satisfy bound `{bound}`",
                        parameter.name
                    )));
                }
            }
        }

        let object = self.allocate_object(
            class.id,
            type_arguments,
            class.properties.len(),
            instruction.span,
        )?;
        for property in &class.properties {
            let Some(initializer) = &property.default else {
                continue;
            };
            let declaring_instance =
                runtime_instantiation_for_class(self.program, &instance, property.declaring_class);
            let declaration = &self.program.classes[property.declaring_class.0 as usize];
            let ty = substitute_runtime_type(
                &substitute_runtime_type(&property.ty, Some(&instance), &class),
                declaring_instance.as_ref(),
                declaration,
            );
            object
                .set_property(
                    property.id,
                    self.materialize_constant(initializer, &ty, instruction.span)?,
                )
                .map_err(|kind| runtime(kind, instruction.span))?;
        }

        let Some(method) = class
            .methods
            .iter()
            .find(|method| method.name == "__construct")
        else {
            if arguments.is_empty() {
                return Ok(object);
            }
            return Err(fail(format!("class `{name}` has no constructor arguments")));
        };
        let Some(callee) = method.callee else {
            return Err(fail(format!("class `{name}` has no concrete constructor")));
        };
        if !runtime_member_accessible(
            self.program,
            calling_function.owner,
            method.declaring_class,
            method.visibility,
        ) {
            return Err(fail(format!(
                "constructor for `{name}` is not accessible here"
            )));
        }
        let declaring_instance =
            runtime_instantiation_for_class(self.program, &instance, method.declaring_class);
        let declaration = &self.program.classes[method.declaring_class.0 as usize];
        let parameters = method
            .parameters
            .iter()
            .map(|parameter| {
                let mut parameter = parameter.clone();
                parameter.ty = substitute_runtime_type(
                    &substitute_runtime_type(&parameter.ty, Some(&instance), &class),
                    declaring_instance.as_ref(),
                    declaration,
                );
                parameter
            })
            .collect::<Vec<_>>();
        let variadic = parameters.iter().position(|parameter| parameter.variadic);
        let mut bound = vec![None; parameters.len()];
        let mut variadic_values = Vec::new();
        let mut next = 0;
        let mut named_seen = false;
        for (argument_name, value, static_type, span) in arguments {
            let target = if let Some(argument_name) = argument_name {
                named_seen = true;
                parameters
                    .iter()
                    .position(|parameter| parameter.name == argument_name && !parameter.variadic)
                    .ok_or_else(|| {
                        fail(format!("unknown constructor argument `{argument_name}`"))
                    })?
            } else {
                if named_seen {
                    return Err(fail(
                        "positional constructor argument follows a named argument".to_owned(),
                    ));
                }
                while next < parameters.len() && bound[next].is_some() {
                    next += 1;
                }
                if Some(next) == variadic {
                    let element = &parameters[next].ty;
                    if !runtime_argument_matches(self.program, element, &static_type, &value) {
                        return Err(runtime(
                            RuntimeErrorKind::TypeError(format!(
                                "constructor argument has type {}, expected `{element}`",
                                value.type_name()
                            )),
                            span,
                        ));
                    }
                    variadic_values.push(value);
                    continue;
                }
                if next >= parameters.len() {
                    return Err(fail("too many constructor arguments".to_owned()));
                }
                let target = next;
                next += 1;
                target
            };
            if bound[target].is_some() {
                return Err(fail(format!(
                    "constructor parameter `{}` is bound more than once",
                    parameters[target].name
                )));
            }
            if !runtime_argument_matches(self.program, &parameters[target].ty, &static_type, &value)
            {
                return Err(runtime(
                    RuntimeErrorKind::TypeError(format!(
                        "constructor argument has type {}, expected `{}`",
                        value.type_name(),
                        parameters[target].ty
                    )),
                    span,
                ));
            }
            bound[target] = Some(value);
        }
        for (index, parameter) in parameters.iter().enumerate() {
            if parameter.variadic {
                bound[index] = Some(
                    Value::try_vector(parameter.ty.clone(), std::mem::take(&mut variadic_values))
                        .map_err(|kind| runtime(kind, instruction.span))?,
                );
            } else if bound[index].is_none() {
                bound[index] = Some(
                    parameter
                        .default
                        .as_ref()
                        .map(|value| {
                            self.materialize_constant(value, &parameter.ty, instruction.span)
                        })
                        .transpose()?
                        .ok_or_else(|| {
                            fail(format!("missing constructor argument `{}`", parameter.name))
                        })?,
                );
            }
        }
        let mut call_arguments = Vec::with_capacity(bound.len() + 1);
        call_arguments.push(object.clone());
        call_arguments.extend(bound.into_iter().map(Option::unwrap));
        self.invoke_callee(
            callee,
            call_arguments,
            depth,
            Some(class.id),
            calling_function,
            instruction,
        )?;
        Ok(object)
    }

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
                let type_arguments = match instruction.ty.as_ref() {
                    Some(Type::Nominal { arguments, .. }) => {
                        let receiver_arguments = function
                            .owner
                            .filter(|_| !function.static_method)
                            .and_then(|_| function.parameters.first())
                            .and_then(|receiver| frame.locals[receiver.0 as usize].as_ref())
                            .and_then(Value::type_arguments)
                            .unwrap_or_default();
                        arguments
                            .iter()
                            .map(|argument| {
                                function.owner.map_or_else(
                                    || argument.clone(),
                                    |owner| {
                                        substitute_type_arguments(
                                            argument,
                                            owner,
                                            receiver_arguments,
                                        )
                                    },
                                )
                            })
                            .collect()
                    }
                    _ => Vec::new(),
                };
                Some(self.allocate_object(
                    class.id,
                    type_arguments,
                    class.properties.len(),
                    instruction.span,
                )?)
            }
            InstructionKind::NewDynamic {
                target,
                type_arguments,
                arguments,
            } => {
                let target = get_register(frame, *target, instruction.span)?;
                let values = arguments
                    .iter()
                    .map(|argument| {
                        Ok((
                            argument.name.clone(),
                            get_register(frame, argument.value, argument.span)?,
                            function.register_types[argument.value.0 as usize].clone(),
                            argument.span,
                        ))
                    })
                    .collect::<Result<Vec<_>, VmError>>()?;
                Some(self.dynamic_new(
                    &target,
                    type_arguments,
                    values,
                    depth,
                    function,
                    instruction,
                )?)
            }
            InstructionKind::CheckedNarrow { value, narrowed } => {
                let value = get_register(frame, *value, instruction.span)?;
                if !value_matches_type(self.program, &value, narrowed) {
                    return Err(runtime(
                        RuntimeErrorKind::TypeError(format!(
                            "value of type {} failed checked narrowing to `{narrowed}`",
                            value.type_name()
                        )),
                        instruction.span,
                    ));
                }
                Some(value)
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
            Callee::Builtin(builtin) => {
                self.execute_builtin(builtin, arguments, depth, calling_function, instruction)
            }
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
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        match builtin {
            Builtin::IsString => Ok(Value::bool(arguments[0].as_bytes().is_some())),
            Builtin::IsInt => Ok(Value::bool(arguments[0].as_int().is_some())),
            Builtin::IsFloat => Ok(Value::bool(arguments[0].as_float().is_some())),
            Builtin::IsNull => Ok(Value::bool(arguments[0].is_null())),
            Builtin::IsNumeric => Ok(Value::bool(
                arguments[0].as_int().is_some()
                    || arguments[0].as_float().is_some()
                    || arguments[0].as_bytes().is_some_and(is_numeric_string),
            )),
            Builtin::IsVector => Ok(Value::bool(arguments[0].vector_values().is_some())),
            Builtin::IsMap => Ok(Value::bool(arguments[0].map_entries().is_some())),
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
            Builtin::Reflection(operation) => {
                self.execute_reflection(operation, &arguments, depth, calling_function, instruction)
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn execute_reflection(
        &mut self,
        operation: ReflectionBuiltin,
        arguments: &[Value],
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        use ReflectionBuiltin as R;

        let span = instruction.span;
        match operation {
            R::ClassConstruct
            | R::FunctionConstruct
            | R::MethodConstruct
            | R::PropertyConstruct
            | R::ParameterConstruct => {
                self.execute_reflection_constructor(operation, arguments, span)
            }
            R::TypeAllowsNull
            | R::TypeGetDisplayName
            | R::TypeEquals
            | R::TypeIsAssignableFrom
            | R::NamedTypeGetName
            | R::NamedTypeIsBuiltin
            | R::NamedTypeIsTypeParameter
            | R::NamedTypeGetTypeArguments
            | R::UnionTypeGetTypes => self.execute_reflection_type(operation, arguments, span),
            R::ClassGetName
            | R::ClassGetShortName
            | R::ClassGetNamespaceName
            | R::ClassGetModuleName
            | R::ClassGetType
            | R::ClassIsAbstract
            | R::ClassIsFinal
            | R::ClassIsInterface
            | R::ClassIsTrait
            | R::ClassIsInternal
            | R::ClassIsUserDefined
            | R::ClassIsInstantiable
            | R::ClassGetParentClass
            | R::ClassGetInterfaces
            | R::ClassGetTraits
            | R::ClassGetConstructor
            | R::ClassGetDeclaredMethods
            | R::ClassGetDeclaredMethod
            | R::ClassGetMethods
            | R::ClassGetMethod
            | R::ClassHasMethod
            | R::ClassGetDeclaredProperties
            | R::ClassGetDeclaredProperty
            | R::ClassGetProperties
            | R::ClassGetProperty
            | R::ClassHasProperty => self.execute_reflection_class(operation, arguments, span),
            R::ClassNewInstanceArgs => {
                self.reflection_new_instance(arguments, depth, calling_function, instruction)
            }
            R::FunctionInvokeArgs | R::MethodInvokeArgs => {
                self.reflection_invoke(operation, arguments, depth, calling_function, instruction)
            }
            R::CallableGetName
            | R::CallableGetShortName
            | R::CallableGetNamespaceName
            | R::CallableGetModuleName
            | R::CallableGetNumberOfParameters
            | R::CallableGetNumberOfRequiredParameters
            | R::CallableGetParameters
            | R::CallableGetReturnType
            | R::CallableIsVariadic
            | R::CallableIsInternal
            | R::CallableIsUserDefined
            | R::MethodGetDeclaringClass
            | R::MethodGetOriginTrait
            | R::MethodGetOriginMethod
            | R::MethodIsPublic
            | R::MethodIsProtected
            | R::MethodIsPrivate
            | R::MethodIsStatic
            | R::MethodIsAbstract
            | R::MethodIsFinal
            | R::MethodIsConstructor => {
                self.execute_reflection_callable(operation, arguments, span)
            }
            R::PropertyGetName
            | R::PropertyGetDeclaringClass
            | R::PropertyGetOriginTrait
            | R::PropertyGetType
            | R::PropertyHasDefaultValue
            | R::PropertyGetDefaultValue
            | R::PropertyIsPublic
            | R::PropertyIsProtected
            | R::PropertyIsPrivate
            | R::PropertyIsStatic
            | R::PropertyGetValue
            | R::PropertySetValue => self.execute_reflection_property(operation, arguments, span),
            R::ParameterGetName
            | R::ParameterGetPosition
            | R::ParameterGetType
            | R::ParameterGetDeclaringFunction
            | R::ParameterIsDefaultValueAvailable
            | R::ParameterGetDefaultValue
            | R::ParameterIsOptional
            | R::ParameterIsVariadic => {
                self.execute_reflection_parameter(operation, arguments, span)
            }
        }
    }

    fn execute_reflection_constructor(
        &self,
        operation: ReflectionBuiltin,
        arguments: &[Value],
        span: Span,
    ) -> Result<Value, VmError> {
        use ReflectionBuiltin as R;

        let value = match operation {
            R::ClassConstruct => {
                let (class, ty) = self.reflection_class_target(&arguments[1], span)?;
                ReflectionValue::Class { class, ty }
            }
            R::FunctionConstruct => {
                ReflectionValue::Function(self.reflection_function_target(&arguments[1], span)?)
            }
            R::MethodConstruct => {
                let (class, ty) = self.reflection_class_target(&arguments[1], span)?;
                let name = self.reflection_name(&arguments[2], "method", span)?;
                let metadata = &self.program.classes[class.0 as usize];
                let Some(index) = metadata
                    .methods
                    .iter()
                    .position(|method| method.name == name)
                else {
                    return Err(self.reflection_error("method does not exist", span));
                };
                ReflectionValue::Method {
                    class,
                    index: u32::try_from(index).expect("method count fits u32"),
                    declared: false,
                    type_arguments: descriptor_type_arguments(ty.as_ref()),
                }
            }
            R::PropertyConstruct => {
                let (class, ty) = self.reflection_class_target(&arguments[1], span)?;
                let name = self.reflection_name(&arguments[2], "property", span)?;
                let metadata = &self.program.classes[class.0 as usize];
                let Some(property) = metadata.properties.iter().rev().find(|property| {
                    property.name == name
                        && (property.declaring_class == class
                            || property.visibility != Visibility::Private)
                }) else {
                    return Err(self.reflection_error("property does not exist", span));
                };
                ReflectionValue::Property {
                    class,
                    slot: property.id,
                    type_arguments: descriptor_type_arguments(ty.as_ref()),
                }
            }
            R::ParameterConstruct => {
                let callable = if let Some(values) = arguments[1].vector_values() {
                    if values.len() != 2 {
                        return Err(self
                            .type_error("method callables must contain exactly two values", span));
                    }
                    let (class, ty) = self.reflection_class_target(&values[0], span)?;
                    let name = self.reflection_name(&values[1], "method", span)?;
                    let metadata = &self.program.classes[class.0 as usize];
                    let Some(index) = metadata
                        .methods
                        .iter()
                        .position(|method| method.name == name)
                    else {
                        return Err(self.reflection_error("method does not exist", span));
                    };
                    ReflectionCallable::Method {
                        class,
                        index: u32::try_from(index).expect("method count fits u32"),
                        declared: false,
                        type_arguments: descriptor_type_arguments(ty.as_ref()),
                    }
                } else if arguments[1].as_bytes().is_some() {
                    ReflectionCallable::Function(
                        self.reflection_function_target(&arguments[1], span)?,
                    )
                } else {
                    return Err(self.type_error(
                        "function must be a function name or a two-value method callable",
                        span,
                    ));
                };
                let metadata = self.callable_metadata(&callable, span)?;
                let position = if let Some(position) = arguments[2].as_int() {
                    usize::try_from(position).ok()
                } else if arguments[2].as_bytes().is_some() {
                    let name = self.reflection_name(&arguments[2], "parameter", span)?;
                    metadata
                        .parameters
                        .iter()
                        .position(|parameter| parameter.name == name)
                } else {
                    return Err(self.type_error("parameter must be an int or string", span));
                }
                .filter(|position| *position < metadata.parameters.len())
                .ok_or_else(|| self.reflection_error("parameter does not exist", span))?;
                ReflectionValue::Parameter {
                    callable,
                    position: u32::try_from(position).expect("parameter count fits u32"),
                }
            }
            _ => unreachable!(),
        };
        if !arguments[0].initialize_reflection(value) {
            return Err(self.error("reflection descriptor is already initialized", span));
        }
        Ok(Value::NULL)
    }

    fn reflection_class_target(
        &self,
        target: &Value,
        span: Span,
    ) -> Result<(ClassId, Option<Type>), VmError> {
        if let Some(bytes) = target.as_bytes() {
            let name = std::str::from_utf8(bytes)
                .map_err(|_| self.reflection_error("class name is not valid UTF-8", span))?;
            let name = name.strip_prefix('\\').unwrap_or(name);
            let class = self
                .program
                .classes
                .iter()
                .find(|class| class.name == name)
                .ok_or_else(|| self.reflection_error("class does not exist", span))?;
            let ty = class
                .type_parameters
                .is_empty()
                .then(|| Type::Object(class.name.clone()));
            return Ok((class.id, ty));
        }
        let class = target
            .class_id()
            .ok_or_else(|| self.type_error("class target must be an object or string", span))?;
        let metadata = &self.program.classes[class.0 as usize];
        let type_arguments = target.type_arguments().unwrap_or_default();
        let ty = Some(if type_arguments.is_empty() {
            Type::Object(metadata.name.clone())
        } else {
            Type::Nominal {
                name: metadata.name.clone(),
                arguments: type_arguments.to_vec(),
            }
        });
        Ok((class, ty))
    }

    fn reflection_function_target(
        &self,
        target: &Value,
        span: Span,
    ) -> Result<FunctionId, VmError> {
        let name = self.reflection_name(target, "function", span)?;
        self.program
            .functions
            .iter()
            .find(|function| {
                function.owner.is_none()
                    && function.id != self.program.entry
                    && function.name == name
            })
            .map(|function| function.id)
            .ok_or_else(|| self.reflection_error("function does not exist", span))
    }

    fn reflection_name<'value>(
        &self,
        value: &'value Value,
        kind: &str,
        span: Span,
    ) -> Result<&'value str, VmError> {
        let bytes = value
            .as_bytes()
            .ok_or_else(|| self.type_error(&format!("{kind} name must be a string"), span))?;
        let name = std::str::from_utf8(bytes)
            .map_err(|_| self.reflection_error(&format!("{kind} name is not valid UTF-8"), span))?;
        Ok(name.strip_prefix('\\').unwrap_or(name))
    }

    fn execute_reflection_type(
        &self,
        operation: ReflectionBuiltin,
        arguments: &[Value],
        span: Span,
    ) -> Result<Value, VmError> {
        use ReflectionBuiltin as R;
        let ty = self.reflected_type(&arguments[0], span)?.clone();
        match operation {
            R::TypeAllowsNull => Ok(Value::bool(type_allows_null(&ty))),
            R::TypeGetDisplayName => self.bytes(ty.to_string(), span),
            R::TypeEquals => Ok(Value::bool(
                ty == *self.reflected_type(&arguments[1], span)?,
            )),
            R::TypeIsAssignableFrom => Ok(Value::bool(thp_bytecode::type_accepts(
                self.program,
                &ty,
                self.reflected_type(&arguments[1], span)?,
            ))),
            R::NamedTypeGetName => self.bytes(named_type_name(&ty), span),
            R::NamedTypeIsBuiltin => Ok(Value::bool(!matches!(
                ty,
                Type::Object(_) | Type::Nominal { .. } | Type::Parameter { .. }
            ))),
            R::NamedTypeIsTypeParameter => Ok(Value::bool(matches!(ty, Type::Parameter { .. }))),
            R::NamedTypeGetTypeArguments => {
                let arguments = match ty {
                    Type::Vector(element) => vec![*element],
                    Type::Map(key, value) => vec![*key, *value],
                    Type::Nominal { arguments, .. } => arguments,
                    _ => Vec::new(),
                };
                let values = arguments
                    .into_iter()
                    .map(|ty| self.type_descriptor(ty, span))
                    .collect::<Result<Vec<_>, _>>()?;
                self.vector("ReflectionType", values, span)
            }
            R::UnionTypeGetTypes => {
                let Type::Union(types) = ty else {
                    return Err(self.reflection_error("descriptor is not a union type", span));
                };
                let values = types
                    .into_iter()
                    .map(|ty| self.type_descriptor(ty, span))
                    .collect::<Result<Vec<_>, _>>()?;
                self.vector("ReflectionType", values, span)
            }
            _ => unreachable!(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn execute_reflection_class(
        &self,
        operation: ReflectionBuiltin,
        arguments: &[Value],
        span: Span,
    ) -> Result<Value, VmError> {
        use ReflectionBuiltin as R;
        let (class_id, reflected_type) = self.reflected_class(&arguments[0], span)?;
        let class = &self.program.classes[class_id.0 as usize];
        let descriptor_arguments = descriptor_type_arguments(reflected_type.as_ref());
        match operation {
            R::ClassGetName => self.bytes(&class.name, span),
            R::ClassGetShortName => self.bytes(short_name(&class.name), span),
            R::ClassGetNamespaceName => self.bytes(namespace_name(&class.name), span),
            R::ClassGetModuleName => self.bytes(&class.module_name, span),
            R::ClassGetType => {
                reflected_type.map_or(Ok(Value::NULL), |ty| self.type_descriptor(ty, span))
            }
            R::ClassIsAbstract => Ok(Value::bool(class.abstract_class)),
            R::ClassIsFinal => Ok(Value::bool(class.final_class)),
            R::ClassIsInterface => Ok(Value::bool(class.kind == thp_hir::NominalKind::Interface)),
            R::ClassIsTrait => Ok(Value::bool(class.kind == thp_hir::NominalKind::Trait)),
            R::ClassIsInternal => Ok(Value::bool(class.native)),
            R::ClassIsUserDefined => Ok(Value::bool(!class.native)),
            R::ClassIsInstantiable => Ok(Value::bool(
                class.kind == thp_hir::NominalKind::Class
                    && !class.abstract_class
                    && !class.native
                    && (class.type_parameters.is_empty() || reflected_type.is_some()),
            )),
            R::ClassGetParentClass => class.parent.map_or(Ok(Value::NULL), |parent| {
                let parent_class = &self.program.classes[parent.0 as usize];
                let ty = class.parent_type.as_ref().map(|parent_type| {
                    let arguments = parent_type
                        .arguments
                        .iter()
                        .map(|argument| {
                            substitute_type_arguments(argument, class.id, &descriptor_arguments)
                        })
                        .collect::<Vec<_>>();
                    if arguments.is_empty() {
                        Type::Object(parent_class.name.clone())
                    } else {
                        Type::Nominal {
                            name: parent_class.name.clone(),
                            arguments,
                        }
                    }
                });
                self.class_descriptor(parent, ty, span)
            }),
            R::ClassGetInterfaces => {
                let values = class
                    .interface_types
                    .iter()
                    .map(|interface| {
                        let interface_class = &self.program.classes[interface.class.0 as usize];
                        let arguments = interface
                            .arguments
                            .iter()
                            .map(|argument| {
                                substitute_type_arguments(argument, class.id, &descriptor_arguments)
                            })
                            .collect::<Vec<_>>();
                        let ty = if arguments.is_empty() {
                            Type::Object(interface_class.name.clone())
                        } else {
                            Type::Nominal {
                                name: interface_class.name.clone(),
                                arguments,
                            }
                        };
                        self.class_descriptor(interface.class, Some(ty), span)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.vector("ReflectionClass", values, span)
            }
            R::ClassGetTraits => {
                let values = class
                    .traits
                    .iter()
                    .map(|trait_id| self.class_descriptor(*trait_id, None, span))
                    .collect::<Result<Vec<_>, _>>()?;
                self.vector("ReflectionClass", values, span)
            }
            R::ClassGetConstructor => class
                .methods
                .iter()
                .position(|method| method.name == "__construct")
                .map_or(Ok(Value::NULL), |index| {
                    self.method_descriptor(
                        class.id,
                        index,
                        false,
                        descriptor_arguments.clone(),
                        span,
                    )
                }),
            R::ClassGetDeclaredMethods | R::ClassGetMethods => {
                let declared = operation == R::ClassGetDeclaredMethods;
                let indexes = if declared {
                    (0..class.declared_methods.len()).collect::<Vec<_>>()
                } else {
                    effective_method_indexes(self.program, class.id)
                };
                let values = indexes
                    .into_iter()
                    .map(|index| {
                        self.method_descriptor(
                            class.id,
                            index,
                            declared,
                            descriptor_arguments.clone(),
                            span,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.vector("ReflectionMethod", values, span)
            }
            R::ClassGetDeclaredMethod | R::ClassGetMethod | R::ClassHasMethod => {
                let Some(name) = utf8_argument(arguments, 1) else {
                    return if operation == R::ClassHasMethod {
                        Ok(Value::bool(false))
                    } else if operation == R::ClassGetDeclaredMethod {
                        Ok(Value::NULL)
                    } else {
                        Err(self.reflection_error("method does not exist", span))
                    };
                };
                let declared = operation == R::ClassGetDeclaredMethod;
                let methods = if declared {
                    &class.declared_methods
                } else {
                    &class.methods
                };
                let found = methods.iter().position(|method| method.name == name);
                if operation == R::ClassHasMethod {
                    Ok(Value::bool(found.is_some()))
                } else if operation == R::ClassGetMethod && found.is_none() {
                    Err(self.reflection_error("method does not exist", span))
                } else {
                    found.map_or(Ok(Value::NULL), |index| {
                        self.method_descriptor(
                            class.id,
                            index,
                            declared,
                            descriptor_arguments.clone(),
                            span,
                        )
                    })
                }
            }
            R::ClassGetDeclaredProperties | R::ClassGetProperties => {
                let slots = if operation == R::ClassGetDeclaredProperties {
                    class
                        .declared_properties
                        .iter()
                        .map(|property| property.id)
                        .collect()
                } else {
                    effective_property_slots(self.program, class.id)
                };
                let values = slots
                    .into_iter()
                    .map(|slot| {
                        self.property_descriptor(class.id, slot, descriptor_arguments.clone(), span)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.vector("ReflectionProperty", values, span)
            }
            R::ClassGetDeclaredProperty | R::ClassGetProperty | R::ClassHasProperty => {
                let Some(name) = utf8_argument(arguments, 1) else {
                    return if operation == R::ClassHasProperty {
                        Ok(Value::bool(false))
                    } else if operation == R::ClassGetDeclaredProperty {
                        Ok(Value::NULL)
                    } else {
                        Err(self.reflection_error("property does not exist", span))
                    };
                };
                let declared = operation == R::ClassGetDeclaredProperty;
                let found = if declared {
                    class
                        .declared_properties
                        .iter()
                        .find(|property| property.name == name)
                } else {
                    class.properties.iter().rev().find(|property| {
                        property.name == name
                            && (property.declaring_class == class.id
                                || property.visibility != Visibility::Private)
                    })
                };
                if operation == R::ClassHasProperty {
                    Ok(Value::bool(found.is_some()))
                } else if operation == R::ClassGetProperty && found.is_none() {
                    Err(self.reflection_error("property does not exist", span))
                } else {
                    found.map_or(Ok(Value::NULL), |property| {
                        self.property_descriptor(
                            class.id,
                            property.id,
                            descriptor_arguments.clone(),
                            span,
                        )
                    })
                }
            }
            _ => unreachable!(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn execute_reflection_callable(
        &self,
        operation: ReflectionBuiltin,
        arguments: &[Value],
        span: Span,
    ) -> Result<Value, VmError> {
        use ReflectionBuiltin as R;
        let callable = self.reflected_callable(&arguments[0], span)?;
        let metadata = self.callable_metadata(&callable, span)?;
        match operation {
            R::CallableGetName => self.bytes(&metadata.name, span),
            R::CallableGetShortName => self.bytes(short_name(&metadata.name), span),
            R::CallableGetNamespaceName => self.bytes(namespace_name(&metadata.name), span),
            R::CallableGetModuleName => self.bytes(&metadata.module_name, span),
            R::CallableGetNumberOfParameters => Ok(Value::integer(
                i64::try_from(metadata.parameters.len()).unwrap_or(i64::MAX),
            )),
            R::CallableGetNumberOfRequiredParameters => Ok(Value::integer(
                i64::try_from(
                    metadata
                        .parameters
                        .iter()
                        .filter(|parameter| parameter.default.is_none() && !parameter.variadic)
                        .count(),
                )
                .unwrap_or(i64::MAX),
            )),
            R::CallableGetParameters => {
                let values = (0..metadata.parameters.len())
                    .map(|position| {
                        self.reflection_descriptor(
                            "ReflectionParameter",
                            ReflectionValue::Parameter {
                                callable: callable.clone(),
                                position: u32::try_from(position)
                                    .expect("parameter count fits u32"),
                            },
                            span,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.vector("ReflectionParameter", values, span)
            }
            R::CallableGetReturnType => self.type_descriptor(metadata.return_type, span),
            R::CallableIsVariadic => Ok(Value::bool(
                metadata
                    .parameters
                    .iter()
                    .any(|parameter| parameter.variadic),
            )),
            R::CallableIsInternal => Ok(Value::bool(metadata.internal)),
            R::CallableIsUserDefined => Ok(Value::bool(!metadata.internal)),
            R::MethodGetDeclaringClass => self.class_descriptor(
                metadata.declaring_class.expect("method owner"),
                callable_descriptor_type(self.program, &callable, metadata.declaring_class),
                span,
            ),
            R::MethodGetOriginTrait => metadata
                .method
                .as_ref()
                .and_then(|method| method.origin_trait)
                .map_or(Ok(Value::NULL), |origin| {
                    self.class_descriptor(origin, None, span)
                }),
            R::MethodGetOriginMethod => {
                let Some(method) = metadata.method.as_ref() else {
                    return Err(self.reflection_error("descriptor is not a method", span));
                };
                let Some(origin) = method.origin_trait else {
                    return Ok(Value::NULL);
                };
                let origin_class = &self.program.classes[origin.0 as usize];
                origin_class
                    .declared_methods
                    .iter()
                    .position(|candidate| candidate.name == method.origin_name)
                    .map_or(Ok(Value::NULL), |index| {
                        self.method_descriptor(origin, index, true, Vec::new(), span)
                    })
            }
            R::MethodIsPublic | R::MethodIsProtected | R::MethodIsPrivate => {
                let visibility = metadata.method.expect("method operation").visibility;
                Ok(Value::bool(match operation {
                    R::MethodIsPublic => visibility == Visibility::Public,
                    R::MethodIsProtected => visibility == Visibility::Protected,
                    _ => visibility == Visibility::Private,
                }))
            }
            R::MethodIsStatic => Ok(Value::bool(metadata.method.expect("method").static_method)),
            R::MethodIsAbstract => Ok(Value::bool(
                metadata.method.expect("method").abstract_method,
            )),
            R::MethodIsFinal => Ok(Value::bool(metadata.method.expect("method").final_method)),
            R::MethodIsConstructor => Ok(Value::bool(
                metadata.method.expect("method").name == "__construct",
            )),
            _ => unreachable!(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn execute_reflection_property(
        &self,
        operation: ReflectionBuiltin,
        arguments: &[Value],
        span: Span,
    ) -> Result<Value, VmError> {
        use ReflectionBuiltin as R;
        let (table_class, property, type_arguments) =
            self.reflected_property(&arguments[0], span)?;
        let property = property.clone();
        let property_type = substitute_type_arguments(&property.ty, table_class, &type_arguments);
        match operation {
            R::PropertyGetName => self.bytes(&property.name, span),
            R::PropertyGetDeclaringClass => {
                let ty = (table_class == property.declaring_class)
                    .then(|| descriptor_nominal_type(self.program, table_class, &type_arguments));
                self.class_descriptor(property.declaring_class, ty, span)
            }
            R::PropertyGetOriginTrait => property.origin_trait.map_or(Ok(Value::NULL), |origin| {
                self.class_descriptor(origin, None, span)
            }),
            R::PropertyGetType => self.type_descriptor(property_type.clone(), span),
            R::PropertyHasDefaultValue => Ok(Value::bool(property.default.is_some())),
            R::PropertyGetDefaultValue => {
                property.default.as_ref().map_or(Ok(Value::NULL), |value| {
                    self.materialize_constant(value, &property_type, span)
                })
            }
            R::PropertyIsPublic | R::PropertyIsProtected | R::PropertyIsPrivate => {
                Ok(Value::bool(match operation {
                    R::PropertyIsPublic => property.visibility == Visibility::Public,
                    R::PropertyIsProtected => property.visibility == Visibility::Protected,
                    _ => property.visibility == Visibility::Private,
                }))
            }
            R::PropertyIsStatic => Ok(Value::bool(false)),
            R::PropertyGetValue | R::PropertySetValue => {
                let receiver = &arguments[1];
                if receiver.class_id().is_none() {
                    return Err(self.reflection_error("property receiver must be an object", span));
                }
                let expected = descriptor_nominal_type(self.program, table_class, &type_arguments);
                if !value_matches(self.program, receiver, &expected) {
                    return Err(self.reflection_error(
                        "property receiver has the wrong class or generic arguments",
                        span,
                    ));
                }
                if operation == R::PropertyGetValue {
                    receiver
                        .property(property.id)
                        .map_err(|kind| runtime(kind, span))
                } else {
                    let value = &arguments[2];
                    if !value_matches(self.program, value, &property_type) {
                        return Err(self.reflection_error(
                            "assigned value does not match the property type",
                            span,
                        ));
                    }
                    receiver
                        .set_property(property.id, value.clone())
                        .map_err(|kind| runtime(kind, span))?;
                    Ok(Value::NULL)
                }
            }
            _ => unreachable!(),
        }
    }

    fn execute_reflection_parameter(
        &self,
        operation: ReflectionBuiltin,
        arguments: &[Value],
        span: Span,
    ) -> Result<Value, VmError> {
        use ReflectionBuiltin as R;
        let (callable, position) = self.reflected_parameter(&arguments[0], span)?;
        let metadata = self.callable_metadata(&callable, span)?;
        let parameter = metadata
            .parameters
            .get(position as usize)
            .ok_or_else(|| self.reflection_error("parameter is out of bounds", span))?;
        match operation {
            R::ParameterGetName => self.bytes(&parameter.name, span),
            R::ParameterGetPosition => Ok(Value::integer(i64::from(position))),
            R::ParameterGetType => self.type_descriptor(parameter.ty.clone(), span),
            R::ParameterGetDeclaringFunction => match callable {
                ReflectionCallable::Function(function) => self.reflection_descriptor(
                    "ReflectionFunction",
                    ReflectionValue::Function(function),
                    span,
                ),
                ReflectionCallable::Method {
                    class,
                    index,
                    declared,
                    type_arguments,
                } => self.reflection_descriptor(
                    "ReflectionMethod",
                    ReflectionValue::Method {
                        class,
                        index,
                        declared,
                        type_arguments,
                    },
                    span,
                ),
            },
            R::ParameterIsDefaultValueAvailable => Ok(Value::bool(parameter.default.is_some())),
            R::ParameterGetDefaultValue => parameter.default.as_ref().map_or_else(
                || Err(self.reflection_error("parameter has no default value", span)),
                |value| self.materialize_constant(value, &parameter.ty, span),
            ),
            R::ParameterIsOptional => Ok(Value::bool(
                parameter.default.is_some() || parameter.variadic,
            )),
            R::ParameterIsVariadic => Ok(Value::bool(parameter.variadic)),
            _ => unreachable!(),
        }
    }

    fn reflection_descriptor(
        &self,
        class_name: &str,
        value: ReflectionValue,
        span: Span,
    ) -> Result<Value, VmError> {
        Value::try_reflection(self.class_named(class_name, span)?, value)
            .map_err(|kind| runtime(kind, span))
    }

    fn type_descriptor(&self, ty: Type, span: Span) -> Result<Value, VmError> {
        let class = if matches!(ty, Type::Union(_)) {
            "ReflectionUnionType"
        } else {
            "ReflectionNamedType"
        };
        self.reflection_descriptor(class, ReflectionValue::Type(ty), span)
    }

    fn class_descriptor(
        &self,
        class: ClassId,
        ty: Option<Type>,
        span: Span,
    ) -> Result<Value, VmError> {
        self.reflection_descriptor(
            "ReflectionClass",
            ReflectionValue::Class { class, ty },
            span,
        )
    }

    fn method_descriptor(
        &self,
        class: ClassId,
        index: usize,
        declared: bool,
        type_arguments: Vec<Type>,
        span: Span,
    ) -> Result<Value, VmError> {
        self.reflection_descriptor(
            "ReflectionMethod",
            ReflectionValue::Method {
                class,
                index: u32::try_from(index).expect("method count fits u32"),
                declared,
                type_arguments,
            },
            span,
        )
    }

    fn property_descriptor(
        &self,
        class: ClassId,
        slot: thp_hir::PropertyId,
        type_arguments: Vec<Type>,
        span: Span,
    ) -> Result<Value, VmError> {
        self.reflection_descriptor(
            "ReflectionProperty",
            ReflectionValue::Property {
                class,
                slot,
                type_arguments,
            },
            span,
        )
    }

    fn reflected_type<'value>(
        &self,
        value: &'value Value,
        span: Span,
    ) -> Result<&'value Type, VmError> {
        let Some(ReflectionValue::Type(ty)) = value.reflection() else {
            return Err(self.reflection_error("descriptor is not a reflected type", span));
        };
        Ok(ty)
    }

    fn reflected_class(
        &self,
        value: &Value,
        span: Span,
    ) -> Result<(ClassId, Option<Type>), VmError> {
        let Some(ReflectionValue::Class { class, ty }) = value.reflection() else {
            return Err(self.reflection_error("descriptor is not a reflected class", span));
        };
        Ok((*class, ty.clone()))
    }

    fn reflected_callable(&self, value: &Value, span: Span) -> Result<ReflectionCallable, VmError> {
        match value.reflection() {
            Some(ReflectionValue::Function(function)) => {
                Ok(ReflectionCallable::Function(*function))
            }
            Some(ReflectionValue::Method {
                class,
                index,
                declared,
                type_arguments,
            }) => Ok(ReflectionCallable::Method {
                class: *class,
                index: *index,
                declared: *declared,
                type_arguments: type_arguments.clone(),
            }),
            _ => Err(self.reflection_error("descriptor is not callable", span)),
        }
    }

    fn reflected_method(
        &self,
        callable: &ReflectionCallable,
        span: Span,
    ) -> Result<Method, VmError> {
        let ReflectionCallable::Method {
            class,
            index,
            declared,
            ..
        } = callable
        else {
            return Err(self.reflection_error("descriptor is not a method", span));
        };
        let class = &self.program.classes[class.0 as usize];
        let methods = if *declared {
            &class.declared_methods
        } else {
            &class.methods
        };
        methods
            .get(*index as usize)
            .cloned()
            .ok_or_else(|| self.reflection_error("method descriptor is invalid", span))
    }

    fn reflected_property(
        &self,
        value: &Value,
        span: Span,
    ) -> Result<(ClassId, Property, Vec<Type>), VmError> {
        let Some(ReflectionValue::Property {
            class,
            slot,
            type_arguments,
        }) = value.reflection()
        else {
            return Err(self.reflection_error("descriptor is not a property", span));
        };
        let property = self.program.classes[class.0 as usize]
            .properties
            .get(slot.0 as usize)
            .cloned()
            .ok_or_else(|| self.reflection_error("property descriptor is invalid", span))?;
        Ok((*class, property, type_arguments.clone()))
    }

    fn reflected_parameter(
        &self,
        value: &Value,
        span: Span,
    ) -> Result<(ReflectionCallable, u32), VmError> {
        let Some(ReflectionValue::Parameter { callable, position }) = value.reflection() else {
            return Err(self.reflection_error("descriptor is not a parameter", span));
        };
        Ok((callable.clone(), *position))
    }

    fn callable_metadata(
        &self,
        callable: &ReflectionCallable,
        span: Span,
    ) -> Result<CallableMetadata, VmError> {
        match callable {
            ReflectionCallable::Function(function) => {
                let function = self
                    .program
                    .functions
                    .get(function.0 as usize)
                    .ok_or_else(|| self.reflection_error("function descriptor is invalid", span))?;
                Ok(CallableMetadata {
                    name: function.name.clone(),
                    module_name: function.module_name.clone(),
                    parameters: function.parameter_metadata.clone(),
                    return_type: function.return_type.clone(),
                    internal: false,
                    declaring_class: None,
                    method: None,
                })
            }
            ReflectionCallable::Method {
                class,
                type_arguments,
                ..
            } => {
                let method = self.reflected_method(callable, span)?;
                let parameters = method
                    .parameters
                    .iter()
                    .cloned()
                    .map(|mut parameter| {
                        parameter.ty =
                            substitute_type_arguments(&parameter.ty, *class, type_arguments);
                        parameter
                    })
                    .collect();
                let declaring = &self.program.classes[method.declaring_class.0 as usize];
                Ok(CallableMetadata {
                    name: method.name.clone(),
                    module_name: declaring.module_name.clone(),
                    parameters,
                    return_type: substitute_type_arguments(
                        &method.return_type,
                        *class,
                        type_arguments,
                    ),
                    internal: declaring.native,
                    declaring_class: Some(method.declaring_class),
                    method: Some(method),
                })
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn bytes(&self, value: impl AsRef<str>, span: Span) -> Result<Value, VmError> {
        Value::try_bytes(value.as_ref().as_bytes().to_vec()).map_err(|kind| runtime(kind, span))
    }

    #[allow(clippy::unused_self)]
    fn vector(&self, class_name: &str, values: Vec<Value>, span: Span) -> Result<Value, VmError> {
        Value::try_vector(Type::Object(class_name.to_owned()), values)
            .map_err(|kind| runtime(kind, span))
    }

    fn allocate_object(
        &self,
        class: ClassId,
        type_arguments: Vec<Type>,
        property_count: usize,
        span: Span,
    ) -> Result<Value, VmError> {
        let class_name = self.program.classes[class.0 as usize].name.as_str();
        if matches!(
            class_name,
            "ReflectionClass"
                | "ReflectionFunction"
                | "ReflectionMethod"
                | "ReflectionProperty"
                | "ReflectionParameter"
        ) {
            Value::try_uninitialized_reflection(class)
        } else if is_instance_of_name(self.program, class, "Throwable") {
            Value::try_throwable_object(class, property_count)
        } else {
            Value::try_typed_object(class, type_arguments, property_count)
        }
        .map_err(|kind| runtime(kind, span))
    }

    #[allow(clippy::self_only_used_in_recursion)]
    fn materialize_constant(
        &self,
        constant: &ConstantValue,
        expected: &Type,
        span: Span,
    ) -> Result<Value, VmError> {
        match constant {
            ConstantValue::Int(value) => Ok(Value::integer(*value)),
            ConstantValue::Float(value) => Ok(Value::float(*value)),
            ConstantValue::Bool(value) => Ok(Value::bool(*value)),
            ConstantValue::Null => Ok(Value::NULL),
            ConstantValue::String(value) => {
                Value::try_bytes(value.clone()).map_err(|kind| runtime(kind, span))
            }
            ConstantValue::Vector(values) => {
                let element = match expected {
                    Type::Vector(element) => Some(element.as_ref()),
                    Type::Union(members) => members.iter().find_map(|member| match member {
                        Type::Vector(element) => Some(element.as_ref()),
                        _ => None,
                    }),
                    _ => None,
                }
                .cloned()
                .unwrap_or(Type::Mixed);
                let values = values
                    .iter()
                    .map(|value| self.materialize_constant(value, &element, span))
                    .collect::<Result<Vec<_>, _>>()?;
                Value::try_vector(element, values).map_err(|kind| runtime(kind, span))
            }
            ConstantValue::Map(entries) => {
                let (key_type, value_type) = match expected {
                    Type::Map(key, value) => Some((key.as_ref(), value.as_ref())),
                    Type::Union(members) => members.iter().find_map(|member| match member {
                        Type::Map(key, value) => Some((key.as_ref(), value.as_ref())),
                        _ => None,
                    }),
                    _ => None,
                }
                .map_or((Type::Mixed, Type::Mixed), |(key, value)| {
                    (key.clone(), value.clone())
                });
                let entries = entries
                    .iter()
                    .map(|(key, value)| {
                        Ok((
                            self.materialize_constant(key, &key_type, span)?,
                            self.materialize_constant(value, &value_type, span)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, VmError>>()?;
                Value::try_map(key_type, value_type, entries).map_err(|kind| runtime(kind, span))
            }
        }
    }

    fn reflection_invoke(
        &mut self,
        operation: ReflectionBuiltin,
        arguments: &[Value],
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        let span = instruction.span;
        let callable = self.reflected_callable(&arguments[0], span)?;
        let metadata = self.callable_metadata(&callable, span)?;
        let supplied = if operation == ReflectionBuiltin::FunctionInvokeArgs {
            &arguments[1]
        } else {
            &arguments[2]
        };
        let mut bound = self.bind_reflection_arguments(supplied, &metadata.parameters, span)?;
        let (callee, called_class) = match &callable {
            ReflectionCallable::Function(function) => (Callee::Function(*function), None),
            ReflectionCallable::Method {
                class,
                type_arguments,
                ..
            } => {
                let method = metadata.method.as_ref().expect("method metadata");
                let Some(callee) = method.callee else {
                    return Err(self.reflection_error("abstract methods cannot be invoked", span));
                };
                if method.static_method {
                    (callee, Some(method.declaring_class))
                } else {
                    let receiver = &arguments[1];
                    let Some(actual) = receiver.class_id() else {
                        return Err(self.reflection_error(
                            "instance method invocation requires an object receiver",
                            span,
                        ));
                    };
                    let expected = descriptor_nominal_type(self.program, *class, type_arguments);
                    if !value_matches(self.program, receiver, &expected) {
                        return Err(
                            self.reflection_error("method receiver has the wrong type", span)
                        );
                    }
                    bound.insert(0, receiver.clone());
                    (callee, Some(actual))
                }
            }
        };
        let result = self.invoke_callee(
            callee,
            bound,
            depth,
            called_class,
            calling_function,
            instruction,
        )?;
        if metadata.return_type != Type::Void
            && !value_matches(self.program, &result, &metadata.return_type)
        {
            return Err(
                self.reflection_error("invoked callable returned a value of the wrong type", span)
            );
        }
        Ok(result)
    }

    fn reflection_new_instance(
        &mut self,
        arguments: &[Value],
        depth: usize,
        calling_function: &Function,
        instruction: &Instruction,
    ) -> Result<Value, VmError> {
        let span = instruction.span;
        let (class_id, reflected_type) = self.reflected_class(&arguments[0], span)?;
        let class = self.program.classes[class_id.0 as usize].clone();
        let type_arguments = descriptor_type_arguments(reflected_type.as_ref());
        if class.kind != thp_hir::NominalKind::Class || class.abstract_class {
            return Err(self.error("reflected type is not a concrete class", span));
        }
        if class.native || type_arguments.len() != class.type_parameters.len() {
            return Err(self.error("reflected type is not instantiable", span));
        }
        let Some(method) = class
            .methods
            .iter()
            .find(|method| method.name == "__construct")
            .cloned()
        else {
            return Err(self.reflection_error("class has no constructor", span));
        };
        if method.visibility != Visibility::Public {
            return Err(self.reflection_error("constructor is not public", span));
        }
        let parameters = method
            .parameters
            .iter()
            .cloned()
            .map(|mut parameter| {
                parameter.ty = substitute_type_arguments(&parameter.ty, class.id, &type_arguments);
                parameter
            })
            .collect::<Vec<_>>();
        let mut bound = self.bind_reflection_arguments(&arguments[1], &parameters, span)?;
        let Some(callee) = method.callee else {
            return Err(self.reflection_error("constructor is abstract", span));
        };
        let object = self.allocate_object(
            class.id,
            type_arguments.clone(),
            class.properties.len(),
            span,
        )?;
        for property in &class.properties {
            if let Some(default) = &property.default {
                let expected = substitute_type_arguments(&property.ty, class.id, &type_arguments);
                let value = self.materialize_constant(default, &expected, span)?;
                object
                    .set_property(property.id, value)
                    .map_err(|kind| runtime(kind, span))?;
            }
        }
        bound.insert(0, object.clone());
        self.invoke_callee(
            callee,
            bound,
            depth,
            Some(class.id),
            calling_function,
            instruction,
        )?;
        Ok(object)
    }

    fn bind_reflection_arguments(
        &self,
        supplied: &Value,
        parameters: &[ParameterMetadata],
        span: Span,
    ) -> Result<Vec<Value>, VmError> {
        let variadic = parameters.iter().position(|parameter| parameter.variadic);
        let mut bound = vec![None; parameters.len()];
        if let Some(values) = supplied.vector_values() {
            let fixed = variadic.unwrap_or(parameters.len());
            if values.len() > fixed && variadic.is_none() {
                return Err(self.argument_count_error("too many positional arguments", span));
            }
            for (index, value) in values.iter().take(fixed).enumerate() {
                bound[index] = Some(value.clone());
            }
            if let Some(index) = variadic {
                let values = values.iter().skip(index).cloned().collect::<Vec<_>>();
                for value in &values {
                    if !value_matches(self.program, value, &parameters[index].ty) {
                        return Err(self.type_error("variadic argument has the wrong type", span));
                    }
                }
                bound[index] = Some(
                    Value::try_vector(parameters[index].ty.clone(), values)
                        .map_err(|kind| runtime(kind, span))?,
                );
            }
        } else if let Some(entries) = supplied.map_entries() {
            for (key, value) in entries {
                let Some(name) = key
                    .as_bytes()
                    .and_then(|name| std::str::from_utf8(name).ok())
                else {
                    return Err(self.type_error("named argument keys must be UTF-8 strings", span));
                };
                let Some(index) = parameters
                    .iter()
                    .position(|parameter| parameter.name == name)
                else {
                    return Err(self.error("unknown named argument", span));
                };
                if parameters[index].variadic {
                    return Err(
                        self.error("named arguments cannot target a variadic parameter", span)
                    );
                }
                if bound[index].replace(value.clone()).is_some() {
                    return Err(self.error("duplicate named argument", span));
                }
            }
            if let Some(index) = variadic {
                bound[index] = Some(
                    Value::try_vector(parameters[index].ty.clone(), Vec::new())
                        .map_err(|kind| runtime(kind, span))?,
                );
            }
        } else {
            return Err(self.type_error(
                "arguments must be vector<mixed> or map<string, mixed>",
                span,
            ));
        }
        for (index, parameter) in parameters.iter().enumerate() {
            if bound[index].is_none() {
                let Some(default) = &parameter.default else {
                    return Err(self.argument_count_error("missing required argument", span));
                };
                bound[index] = Some(self.materialize_constant(default, &parameter.ty, span)?);
            }
            if !parameter.variadic
                && !value_matches(
                    self.program,
                    bound[index].as_ref().expect("argument bound"),
                    &parameter.ty,
                )
            {
                return Err(self.type_error("argument has the wrong type", span));
            }
        }
        Ok(bound.into_iter().flatten().collect())
    }

    fn reflection_error(&self, message: &str, span: Span) -> VmError {
        self.native_exception(
            "ReflectionException",
            message.as_bytes().to_vec(),
            None,
            0,
            span,
        )
    }

    fn type_error(&self, message: &str, span: Span) -> VmError {
        self.native_exception("TypeError", message.as_bytes().to_vec(), None, 0, span)
    }

    fn argument_count_error(&self, message: &str, span: Span) -> VmError {
        self.native_exception(
            "ArgumentCountError",
            message.as_bytes().to_vec(),
            None,
            0,
            span,
        )
    }

    fn error(&self, message: &str, span: Span) -> VmError {
        self.native_exception("Error", message.as_bytes().to_vec(), None, 0, span)
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

#[derive(Clone)]
struct CallableMetadata {
    name: String,
    module_name: String,
    parameters: Vec<ParameterMetadata>,
    return_type: Type,
    internal: bool,
    declaring_class: Option<ClassId>,
    method: Option<Method>,
}

fn utf8_argument(arguments: &[Value], index: usize) -> Option<&str> {
    arguments
        .get(index)
        .and_then(Value::as_bytes)
        .and_then(|value| std::str::from_utf8(value).ok())
}

fn short_name(name: &str) -> &str {
    name.rsplit_once('\\').map_or(name, |(_, short)| short)
}

fn namespace_name(name: &str) -> &str {
    name.rsplit_once('\\')
        .map_or("", |(namespace, _)| namespace)
}

fn named_type_name(ty: &Type) -> String {
    match ty {
        Type::Vector(_) => "vector".to_owned(),
        Type::Map(_, _) => "map".to_owned(),
        Type::Nominal { name, .. } | Type::Object(name) | Type::Parameter { name, .. } => {
            name.clone()
        }
        other => other.to_string(),
    }
}

fn type_allows_null(ty: &Type) -> bool {
    ty == &Type::Null
        || matches!(ty, Type::Union(members) if members.iter().any(type_allows_null))
        || ty == &Type::Mixed
}

fn descriptor_type_arguments(ty: Option<&Type>) -> Vec<Type> {
    match ty {
        Some(Type::Nominal { arguments, .. }) => arguments.clone(),
        _ => Vec::new(),
    }
}

fn descriptor_nominal_type(program: &Program, class: ClassId, arguments: &[Type]) -> Type {
    let name = program.classes[class.0 as usize].name.clone();
    if arguments.is_empty() {
        Type::Object(name)
    } else {
        Type::Nominal {
            name,
            arguments: arguments.to_vec(),
        }
    }
}

fn callable_descriptor_type(
    program: &Program,
    callable: &ReflectionCallable,
    declaring_class: Option<ClassId>,
) -> Option<Type> {
    let ReflectionCallable::Method {
        class,
        type_arguments,
        ..
    } = callable
    else {
        return None;
    };
    (*class == declaring_class?).then(|| descriptor_nominal_type(program, *class, type_arguments))
}

fn substitute_type_arguments(ty: &Type, owner: ClassId, arguments: &[Type]) -> Type {
    match ty {
        Type::Parameter { id, .. } if id.owner == owner => arguments
            .get(id.index as usize)
            .cloned()
            .unwrap_or_else(|| ty.clone()),
        Type::Vector(element) => Type::Vector(Box::new(substitute_type_arguments(
            element, owner, arguments,
        ))),
        Type::Map(key, value) => Type::Map(
            Box::new(substitute_type_arguments(key, owner, arguments)),
            Box::new(substitute_type_arguments(value, owner, arguments)),
        ),
        Type::Union(members) => Type::Union(
            members
                .iter()
                .map(|member| substitute_type_arguments(member, owner, arguments))
                .collect(),
        ),
        Type::Nominal {
            name,
            arguments: nested,
        } => Type::Nominal {
            name: name.clone(),
            arguments: nested
                .iter()
                .map(|argument| substitute_type_arguments(argument, owner, arguments))
                .collect(),
        },
        _ => ty.clone(),
    }
}

fn effective_method_indexes(program: &Program, class_id: ClassId) -> Vec<usize> {
    let class = &program.classes[class_id.0 as usize];
    let mut names = Vec::<String>::new();
    for method in &class.declared_methods {
        if !names.contains(&method.name) {
            names.push(method.name.clone());
        }
    }
    let mut parent = class.parent;
    while let Some(parent_id) = parent {
        let ancestor = &program.classes[parent_id.0 as usize];
        for method in &ancestor.declared_methods {
            if method.visibility != Visibility::Private && !names.contains(&method.name) {
                names.push(method.name.clone());
            }
        }
        parent = ancestor.parent;
    }
    for method in &class.methods {
        if !names.contains(&method.name) {
            names.push(method.name.clone());
        }
    }
    names
        .iter()
        .filter_map(|name| class.methods.iter().position(|method| method.name == *name))
        .collect()
}

fn effective_property_slots(program: &Program, class_id: ClassId) -> Vec<thp_hir::PropertyId> {
    let class = &program.classes[class_id.0 as usize];
    let mut names = Vec::<String>::new();
    let mut slots = Vec::new();
    for property in &class.declared_properties {
        names.push(property.name.clone());
        slots.push(property.id);
    }
    let mut parent = class.parent;
    while let Some(parent_id) = parent {
        let ancestor = &program.classes[parent_id.0 as usize];
        for property in &ancestor.declared_properties {
            if property.visibility != Visibility::Private
                && !names.contains(&property.name)
                && let Some(effective) = class.properties.iter().find(|candidate| {
                    candidate.name == property.name
                        && candidate.declaring_class == property.declaring_class
                })
            {
                names.push(property.name.clone());
                slots.push(effective.id);
            }
        }
        parent = ancestor.parent;
    }
    slots
}

fn value_type(program: &Program, value: &Value) -> Option<Type> {
    if value.is_null() {
        Some(Type::Null)
    } else if value.as_int().is_some() {
        Some(Type::Int)
    } else if value.as_float().is_some() {
        Some(Type::Float)
    } else if value.as_bool().is_some() {
        Some(Type::Bool)
    } else if value.as_bytes().is_some() {
        Some(Type::String)
    } else if let Some((first, second)) = value.collection_types() {
        Some(match second {
            Some(value) => Type::Map(Box::new(first), Box::new(value)),
            None => Type::Vector(Box::new(first)),
        })
    } else {
        let class = value.class_id()?;
        let metadata = &program.classes[class.0 as usize];
        let arguments = value.type_arguments().unwrap_or_default();
        Some(if arguments.is_empty() {
            Type::Object(metadata.name.clone())
        } else {
            Type::Nominal {
                name: metadata.name.clone(),
                arguments: arguments.to_vec(),
            }
        })
    }
}

fn value_matches(program: &Program, value: &Value, expected: &Type) -> bool {
    value_type(program, value)
        .is_some_and(|actual| thp_bytecode::type_accepts(program, expected, &actual))
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

fn is_numeric_string(bytes: &[u8]) -> bool {
    fn whitespace(byte: u8) -> bool {
        matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
    }
    if !bytes.is_ascii() {
        return false;
    }
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && whitespace(bytes[start]) {
        start += 1;
    }
    while end > start && whitespace(bytes[end - 1]) {
        end -= 1;
    }
    let bytes = &bytes[start..end];
    let mut index = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let before = index;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    let mut digits = index - before;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        digits += index - fraction;
    }
    if digits == 0 {
        return false;
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if exponent == index {
            return false;
        }
    }
    index == bytes.len()
}

fn value_matches_type(program: &Program, value: &Value, expected: &Type) -> bool {
    match expected {
        Type::Mixed => true,
        Type::Int => value.as_int().is_some(),
        Type::Float => value.as_float().is_some(),
        Type::Bool => value.as_bool().is_some(),
        Type::String => value.as_bytes().is_some(),
        Type::Null => value.is_null(),
        Type::Vector(_) => value.vector_values().is_some(),
        Type::Map(_, _) => value.map_entries().is_some(),
        Type::Union(members) => members
            .iter()
            .any(|member| value_matches_type(program, value, member)),
        Type::Object(name) | Type::Nominal { name, .. } => value.class_id().is_some_and(|actual| {
            program
                .classes
                .iter()
                .find(|class| &class.name == name)
                .is_some_and(|expected| is_instance_of(program, actual, expected.id))
        }),
        Type::Parameter { id, .. } => program
            .classes
            .get(id.owner.0 as usize)
            .and_then(|class| class.type_parameters.get(id.index as usize))
            .and_then(|parameter| parameter.bound.as_ref())
            .is_some_and(|bound| value_matches_type(program, value, bound)),
        Type::Void | Type::Never => false,
    }
}

fn descriptor_accepts(program: &Program, expected: &Type, actual: &Type) -> bool {
    expected == &Type::Mixed
        || actual == &Type::Never
        || expected == actual
        || matches!(actual, Type::Parameter { id, .. } if program
            .classes
            .get(id.owner.0 as usize)
            .and_then(|class| class.type_parameters.get(id.index as usize))
            .and_then(|parameter| parameter.bound.as_ref())
            .is_some_and(|bound| descriptor_accepts(program, expected, bound)))
        || matches!(
            actual,
            Type::Union(members)
                if members.iter().all(|member| descriptor_accepts(program, expected, member))
        )
        || matches!(
            expected,
            Type::Union(members)
                if members.iter().any(|member| descriptor_accepts(program, member, actual))
        )
        || match (expected, actual) {
            (Type::Object(expected), Type::Object(actual) | Type::Nominal { name: actual, .. }) => {
                let expected = program.classes.iter().find(|class| &class.name == expected);
                let actual = program.classes.iter().find(|class| &class.name == actual);
                expected.zip(actual).is_some_and(|(expected, actual)| {
                    is_instance_of(program, actual.id, expected.id)
                })
            }
            (
                Type::Nominal {
                    name: expected,
                    arguments: expected_arguments,
                },
                Type::Nominal {
                    name: actual,
                    arguments: actual_arguments,
                },
            ) => {
                let expected_class = program.classes.iter().find(|class| &class.name == expected);
                let actual_class = program.classes.iter().find(|class| &class.name == actual);
                expected_class
                    .zip(actual_class)
                    .is_some_and(|(expected, actual)| {
                        runtime_instantiation_for_class(
                            program,
                            &thp_bytecode::NominalType {
                                class: actual.id,
                                arguments: actual_arguments.clone(),
                            },
                            expected.id,
                        )
                        .is_some_and(|instantiated| instantiated.arguments == *expected_arguments)
                    })
            }
            (Type::Vector(expected), Type::Vector(actual)) => {
                descriptor_accepts(program, expected, actual)
            }
            (Type::Map(expected_key, expected_value), Type::Map(actual_key, actual_value)) => {
                descriptor_accepts(program, expected_key, actual_key)
                    && descriptor_accepts(program, expected_value, actual_value)
            }
            _ => false,
        }
}

fn runtime_argument_matches(
    program: &Program,
    expected: &Type,
    static_type: &Type,
    value: &Value,
) -> bool {
    if !matches!(static_type, Type::Mixed | Type::Union(_)) {
        return descriptor_accepts(program, expected, static_type);
    }
    match expected {
        Type::Nominal { .. } | Type::Parameter { .. } => false,
        Type::Vector(element) if element.as_ref() != &Type::Mixed => false,
        Type::Map(key, value) if key.as_ref() != &Type::Mixed || value.as_ref() != &Type::Mixed => {
            false
        }
        _ => value_matches_type(program, value, expected),
    }
}

fn substitute_runtime_type(
    ty: &Type,
    instantiated: Option<&thp_bytecode::NominalType>,
    declaration: &thp_bytecode::Class,
) -> Type {
    fn apply(
        ty: &Type,
        substitutions: &std::collections::BTreeMap<thp_hir::TypeParameterId, Type>,
    ) -> Type {
        match ty {
            Type::Parameter { id, .. } => {
                substitutions.get(id).cloned().unwrap_or_else(|| ty.clone())
            }
            Type::Vector(element) => Type::Vector(Box::new(apply(element, substitutions))),
            Type::Map(key, value) => Type::Map(
                Box::new(apply(key, substitutions)),
                Box::new(apply(value, substitutions)),
            ),
            Type::Union(members) => Type::Union(
                members
                    .iter()
                    .map(|member| apply(member, substitutions))
                    .collect(),
            ),
            Type::Nominal { name, arguments } => Type::Nominal {
                name: name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| apply(argument, substitutions))
                    .collect(),
            },
            _ => ty.clone(),
        }
    }
    let substitutions = instantiated
        .into_iter()
        .flat_map(|instantiated| {
            declaration
                .type_parameters
                .iter()
                .zip(&instantiated.arguments)
                .map(|(parameter, argument)| (parameter.id, argument.clone()))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    apply(ty, &substitutions)
}

fn runtime_instantiation_for_class(
    program: &Program,
    instantiated: &thp_bytecode::NominalType,
    target: ClassId,
) -> Option<thp_bytecode::NominalType> {
    if instantiated.class == target {
        return Some(instantiated.clone());
    }
    let class = &program.classes[instantiated.class.0 as usize];
    class
        .parent_type
        .iter()
        .chain(&class.interface_types)
        .map(|edge| thp_bytecode::NominalType {
            class: edge.class,
            arguments: edge
                .arguments
                .iter()
                .map(|argument| substitute_runtime_type(argument, Some(instantiated), class))
                .collect(),
        })
        .find_map(|edge| runtime_instantiation_for_class(program, &edge, target))
}

fn runtime_member_accessible(
    program: &Program,
    owner: Option<ClassId>,
    declaring: ClassId,
    visibility: thp_syntax::Visibility,
) -> bool {
    match visibility {
        thp_syntax::Visibility::Public => true,
        thp_syntax::Visibility::Private => owner == Some(declaring),
        thp_syntax::Visibility::Protected => owner
            .is_some_and(|owner| owner == declaring || is_instance_of(program, owner, declaring)),
    }
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

    use super::{
        ExecutionContext, Limits, VmError, execute, execute_captured, execute_to, is_numeric_string,
    };

    #[test]
    fn numeric_string_grammar_covers_boundaries() {
        for accepted in [
            b"0".as_slice(),
            b"+42",
            b"-1.5",
            b".5",
            b"1.",
            b"6.02e23",
            b"\t -2E-3 \r\n",
        ] {
            assert!(is_numeric_string(accepted), "{accepted:?}");
        }
        for rejected in [
            b"".as_slice(),
            b"  ",
            b"+",
            b".",
            b"1e",
            b"1x",
            b"0x10",
            b"0b10",
            b"1_0",
            b"INF",
            b"NAN",
            &[0xff],
        ] {
            assert!(!is_numeric_string(rejected), "{rejected:?}");
        }
    }

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
    fn dynamically_constructs_after_direct_narrowing() {
        let execution = run(r#"<?thp
class Greeter {
    public string $prefix = "hello";
    public function __construct(string $name = "world") { $this->prefix = $this->prefix . " " . $name; }
    public final function message(): string { return $this->prefix; }
}
$class: mixed = "Greeter";
if (is_string($class)) {
    $value = new $class(name: "THP");
    if ($value instanceof Greeter) { echo $value->message() . "\n"; }
}
$other = new ("Greeter")();
if ($other instanceof Greeter) { echo $other->message() . "\n"; }
"#)
        .unwrap();
        assert_eq!(execution.output, b"hello THP\nhello world\n");
    }

    #[test]
    fn guards_match_values_and_php_numeric_strings() {
        let execution = run(r#"<?thp
$value: mixed = " -1.25e+2 ";
if (is_numeric($value)) { var_dump(is_string($value)); }
var_dump(is_numeric("1."));
var_dump(is_numeric(".5"));
var_dump(is_numeric("0x10"));
var_dump(is_numeric("1_0"));
var_dump(is_numeric("INF"));
var_dump(is_int(1));
var_dump(is_float(1.0));
var_dump(is_null(null));
var_dump(is_vector([1]));
var_dump(is_map({"a" => 1}));
"#)
        .unwrap();
        assert_eq!(
            execution.output,
            b"bool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n"
        );
    }

    #[test]
    fn dynamic_new_preserves_order_and_binds_generic_variadics() {
        let execution = run(r#"<?thp
function target(): string { echo "target\n"; return "Box"; }
function argument(): string { echo "argument\n"; return "value"; }
class Box<T> {
    public T $value;
    public function __construct(T $value, string $suffix = "!", int ...$numbers) {
        $this->value = $value;
        echo $suffix . count($numbers) . "\n";
    }
}
$box = new (target())<string>(argument(), "?", 1, 2);
var_dump($box instanceof Box);
class Entity {}
class Named extends Entity {}
class Pair<T extends Entity, U> {
    public function __construct(T $first, U $second) {}
}
$pairClass: string = "Pair";
$pair = new $pairClass<Named>(new Named(), "second");
var_dump($pair instanceof Pair);
class ParentBox<T> { public function __construct(T $value) {} }
class ChildBox<U> extends ParentBox<U> {}
$childClass: string = "ChildBox";
$child = new $childClass<string>("inherited");
var_dump($child instanceof ChildBox);
"#)
        .unwrap();
        assert_eq!(
            execution.output,
            b"target\nargument\n?2\nbool(true)\nbool(true)\nbool(true)\n"
        );
    }

    #[test]
    fn dynamic_new_reports_lookup_constructibility_and_type_failures() {
        let execution = run(r#"<?thp
abstract class AbstractThing {}
interface Contract {}
class Secret { private function __construct() {} }
function attempt(string $name): void {
    try { $value = new $name(); } catch (Error $error) { echo "error\n"; }
}
attempt("Missing");
attempt("AbstractThing");
attempt("Contract");
attempt("Secret");
$invalid: string = "\xff";
try { $value = new $invalid(); } catch (ValueError $error) { echo "utf8\n"; }
"#)
        .unwrap();
        assert_eq!(execution.output, b"error\nerror\nerror\nerror\nutf8\n");

        let error = run(r#"<?thp
class TakesInt { public function __construct(int $value) {} }
$class: string = "TakesInt";
$value = new $class("wrong");
"#)
        .unwrap_err();
        assert!(matches!(
            error,
            VmError::Runtime(RuntimeError {
                kind: RuntimeErrorKind::TypeError(_),
                ..
            })
        ));
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
    fn reflection_materializes_nullable_collection_defaults_with_their_element_type() {
        let execution = run(r#"<?thp
function countValues(?vector<int> $values = [1, 2]): int {
    return 2;
}
function countEntries(?map<string, int> $entries = {"one" => 1}): int {
    return 1;
}
var_dump((new ReflectionFunction("countValues"))->invokeArgs());
var_dump((new ReflectionFunction("countEntries"))->invokeArgs());
"#)
        .unwrap();

        assert_eq!(execution.output, b"int(2)\nint(1)\n");
    }

    #[test]
    fn reflection_constructs_throwable_subclasses_with_throwable_storage() {
        let execution = run(r#"<?thp
class Problem extends Exception {}
$class = new ReflectionClass("Problem");
$problem = $class->newInstanceArgs(["broken"]);
var_dump($class->getMethod("getMessage")->invokeArgs($problem));
"#)
        .unwrap();

        assert_eq!(execution.output, b"string(6) \"broken\"\n");
    }

    #[test]
    fn generic_method_allocations_retain_concrete_type_arguments() {
        let execution = run(r#"<?thp
class Box<T> {
    public T $value;

    public function __construct(T $value) { $this->value = $value; }

    public function copy(): Box<T> { return new Box<T>($this->value); }
}
$copy = (new Box<int>(1))->copy();
var_dump((new ReflectionProperty($copy, "value"))->getValue($copy));
"#)
        .unwrap();

        assert_eq!(execution.output, b"int(1)\n");
    }

    #[test]
    fn reflected_trait_methods_retain_composition_order() {
        let execution = run(r#"<?thp
trait OrderedMethods {
    public function second(): int { return 2; }
    public function first(): int { return 1; }
}
class UsesOrderedMethods { use OrderedMethods; }
foreach ((new ReflectionClass("UsesOrderedMethods"))->getDeclaredMethods() as $method) {
    echo $method->getName() . "\n";
}
"#)
        .unwrap();

        assert_eq!(execution.output, b"second\nfirst\n");
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
