//! Cranelift baseline JIT for THP's verified scalar bytecode subset.

// Calling memory emitted by Cranelift requires an unsafe function-pointer cast.
#![allow(unsafe_code)]

use std::fmt;
use std::io::{self, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};

use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature, condcodes::IntCC, types};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
use thp_bytecode::{Function, InstructionKind, Program, Terminator, VerificationError, verify};
use thp_hir::{Callee, Type};
use thp_mir::Constant;
use thp_runtime::Value;
use thp_syntax::{BinaryOp, UnaryOp};

#[derive(Debug)]
pub enum JitError {
    Verification(VerificationError),
    Unsupported(String),
    Backend(String),
    Output(String),
}

impl fmt::Display for JitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Verification(error) => error.fmt(formatter),
            Self::Unsupported(message) => write!(formatter, "JIT does not support {message}"),
            Self::Backend(message) => write!(formatter, "JIT backend error: {message}"),
            Self::Output(message) => write!(formatter, "host output failed: {message}"),
        }
    }
}

impl std::error::Error for JitError {}

#[derive(Debug)]
pub struct Execution {
    pub result: Value,
    pub output: Vec<u8>,
    pub compiled_functions: usize,
}

#[derive(Debug)]
pub struct StreamingExecution {
    pub result: Value,
    pub output_bytes: u64,
    pub compiled_functions: usize,
}

struct JitContext<'output> {
    output: &'output mut dyn Write,
    output_bytes: u64,
    output_error: Option<String>,
}

unsafe extern "C" fn print_scalar(context: *mut JitContext<'_>, value: i64, kind: u8) {
    // SAFETY: every compiled function receives the live context created by
    // `execute`; generated code forwards that exact pointer to this callback
    // synchronously and never retains it.
    let context = unsafe { &mut *context };
    if context.output_error.is_some() {
        return;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let output = match kind {
            0 => Value::integer(value).output_bytes(),
            1 => Value::bool(value != 0).output_bytes(),
            _ => None,
        }
        .ok_or_else(|| io::Error::other("the compiler emitted an invalid scalar print kind"))?;
        context.output.write_all(&output).map(|()| output.len())
    }));
    match result {
        Ok(Ok(written)) => {
            context.output_bytes = context.output_bytes.saturating_add(written as u64);
        }
        Ok(Err(error)) => context.output_error = Some(error.to_string()),
        Err(_) => context.output_error = Some("output sink panicked".to_owned()),
    }
}

/// Compiles and executes a verified scalar program as native code.
///
/// This first baseline accepts straight-line functions containing scalar
/// constants, locals, direct calls, null/bool tests, and comparisons. Checked
/// arithmetic and heap operations deliberately return `Unsupported`, so
/// `--engine=auto` can preserve semantics by using the VM.
///
/// # Errors
///
/// Returns verification, unsupported-bytecode, target-ISA, and backend errors.
pub fn execute(program: &Program) -> Result<Execution, JitError> {
    let mut capture = FallibleCapture::default();
    let execution = execute_to(program, &mut capture)?;
    Ok(Execution {
        result: execution.result,
        output: capture.bytes,
        compiled_functions: execution.compiled_functions,
    })
}

/// Compiles and executes a verified scalar program while streaming output.
///
/// # Errors
///
/// Returns verification, unsupported-bytecode, backend, or output-sink
/// failures.
pub fn execute_to(
    program: &Program,
    output: &mut dyn Write,
) -> Result<StreamingExecution, JitError> {
    verify(program).map_err(JitError::Verification)?;
    validate_program(program)?;

    let mut flag_builder = settings::builder();
    flag_builder
        .set("opt_level", "speed")
        .map_err(|error| JitError::Backend(error.to_string()))?;
    let flags = settings::Flags::new(flag_builder);
    let isa = cranelift_native::builder()
        .map_err(|error| JitError::Backend(error.to_string()))?
        .finish(flags)
        .map_err(|error| JitError::Backend(error.to_string()))?;
    let mut jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    jit_builder.symbol("thp_print_scalar", print_scalar as *const u8);
    let mut module = JITModule::new(jit_builder);
    let function_ids = declare_functions(&mut module, program)?;
    let print_function = declare_print_function(&mut module)?;
    for function in &program.functions {
        compile_function(&mut module, function, &function_ids, print_function)?;
    }
    module
        .finalize_definitions()
        .map_err(|error| JitError::Backend(error.to_string()))?;

    let entry_index = program.entry.0 as usize;
    let entry_pointer = module.get_finalized_function(function_ids[entry_index]);
    let mut context = JitContext {
        output,
        output_bytes: 0,
        output_error: None,
    };
    // SAFETY: validation requires no source-level entry parameters. The hidden
    // first parameter is a live `JitContext` pointer, all JIT signatures return
    // one i64, definitions are finalized, and the owning module remains alive.
    let entry: unsafe extern "C" fn(*mut JitContext) -> i64 =
        unsafe { std::mem::transmute(entry_pointer) };
    let raw = unsafe { entry(&raw mut context) };
    if let Some(error) = context.output_error {
        return Err(JitError::Output(error));
    }
    Ok(StreamingExecution {
        result: decode_result(&program.functions[entry_index].return_type, raw),
        output_bytes: context.output_bytes,
        compiled_functions: function_ids.len(),
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

pub fn supports(program: &Program) -> bool {
    verify(program).is_ok() && validate_program(program).is_ok()
}

fn validate_program(program: &Program) -> Result<(), JitError> {
    if !program.functions[program.entry.0 as usize]
        .parameters
        .is_empty()
    {
        return Err(JitError::Unsupported(
            "an entry function with parameters".to_owned(),
        ));
    }
    for function in &program.functions {
        if function.blocks.len() != 1 {
            return Err(JitError::Unsupported(
                "control-flow graphs in the baseline tier".to_owned(),
            ));
        }
        for ty in function
            .local_types
            .iter()
            .chain(&function.register_types)
            .chain(std::iter::once(&function.return_type))
        {
            validate_type(ty)?;
        }
        for instruction in &function.blocks[0].instructions {
            match &instruction.kind {
                InstructionKind::Constant(
                    Constant::Integer(_) | Constant::Bool(_) | Constant::Null,
                )
                | InstructionKind::LoadLocal(_)
                | InstructionKind::StoreLocal { .. }
                | InstructionKind::IsNull(_)
                | InstructionKind::Print(_)
                | InstructionKind::Unary {
                    op: UnaryOp::Not, ..
                }
                | InstructionKind::Binary {
                    op:
                        BinaryOp::Equal
                        | BinaryOp::StrictEqual
                        | BinaryOp::NotEqual
                        | BinaryOp::Less
                        | BinaryOp::LessEqual
                        | BinaryOp::Greater
                        | BinaryOp::GreaterEqual
                        | BinaryOp::And
                        | BinaryOp::Or,
                    ..
                }
                | InstructionKind::Call {
                    callee: Callee::Function(_),
                    ..
                } => {}
                other => {
                    return Err(JitError::Unsupported(format!("instruction `{other:?}`")));
                }
            }
        }
        if !matches!(function.blocks[0].terminator, Terminator::Return(_)) {
            return Err(JitError::Unsupported(format!(
                "terminator `{:?}`",
                function.blocks[0].terminator
            )));
        }
    }
    Ok(())
}

fn validate_type(ty: &Type) -> Result<(), JitError> {
    if matches!(ty, Type::Int | Type::Bool | Type::Null | Type::Void) {
        Ok(())
    } else {
        Err(JitError::Unsupported(format!("value type `{ty}`")))
    }
}

fn signature(module: &JITModule, function: &Function) -> Signature {
    let mut signature = module.make_signature();
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature.params.extend(
        function
            .parameters
            .iter()
            .map(|_| AbiParam::new(types::I64)),
    );
    signature.returns.push(AbiParam::new(types::I64));
    signature
}

fn declare_print_function(module: &mut JITModule) -> Result<FuncId, JitError> {
    let mut signature = module.make_signature();
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature.params.push(AbiParam::new(types::I64));
    signature.params.push(AbiParam::new(types::I8));
    module
        .declare_function("thp_print_scalar", Linkage::Import, &signature)
        .map_err(|error| JitError::Backend(error.to_string()))
}

fn declare_functions(module: &mut JITModule, program: &Program) -> Result<Vec<FuncId>, JitError> {
    program
        .functions
        .iter()
        .map(|function| {
            module
                .declare_function(
                    &format!("thp_fn_{}", function.id.0),
                    Linkage::Local,
                    &signature(module, function),
                )
                .map_err(|error| JitError::Backend(error.to_string()))
        })
        .collect()
}

#[allow(clippy::too_many_lines)]
fn compile_function(
    module: &mut JITModule,
    function: &Function,
    function_ids: &[FuncId],
    print_function: FuncId,
) -> Result<(), JitError> {
    let mut context = module.make_context();
    context.func.signature = signature(module, function);
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    builder.seal_block(block);
    let context_pointer = builder.block_params(block)[0];

    let locals = function
        .local_types
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let variable =
                Variable::from_u32(u32::try_from(index).expect("verified local count fits u32"));
            builder.declare_var(variable, types::I64);
            variable
        })
        .collect::<Vec<_>>();
    for (index, parameter) in function.parameters.iter().enumerate() {
        builder.def_var(
            locals[parameter.0 as usize],
            builder.block_params(block)[index + 1],
        );
    }

    let mut registers = vec![None; function.register_types.len()];
    for instruction in &function.blocks[0].instructions {
        let value = match &instruction.kind {
            InstructionKind::Constant(constant) => Some(match constant {
                Constant::Integer(value) => builder.ins().iconst(types::I64, *value),
                Constant::Bool(value) => builder.ins().iconst(types::I64, i64::from(*value)),
                Constant::Null => builder.ins().iconst(types::I64, 0),
                _ => unreachable!("unsupported constant rejected"),
            }),
            InstructionKind::LoadLocal(local) => Some(builder.use_var(locals[local.0 as usize])),
            InstructionKind::StoreLocal { local, value } => {
                builder.def_var(locals[local.0 as usize], get_register(&registers, *value)?);
                None
            }
            InstructionKind::Unary {
                op: UnaryOp::Not,
                operand,
            } => {
                let compared =
                    builder
                        .ins()
                        .icmp_imm(IntCC::Equal, get_register(&registers, *operand)?, 0);
                Some(builder.ins().uextend(types::I64, compared))
            }
            InstructionKind::Binary { op, left, right } => Some(lower_binary(
                &mut builder,
                *op,
                get_register(&registers, *left)?,
                get_register(&registers, *right)?,
            )),
            InstructionKind::IsNull(value) => {
                let compared =
                    builder
                        .ins()
                        .icmp_imm(IntCC::Equal, get_register(&registers, *value)?, 0);
                Some(builder.ins().uextend(types::I64, compared))
            }
            InstructionKind::Call { callee, arguments } => {
                let Callee::Function(target) = callee else {
                    unreachable!("builtin call rejected")
                };
                let reference =
                    module.declare_func_in_func(function_ids[target.0 as usize], builder.func);
                let mut arguments = arguments
                    .iter()
                    .map(|argument| get_register(&registers, *argument))
                    .collect::<Result<Vec<_>, _>>()?;
                arguments.insert(0, context_pointer);
                let call = builder.ins().call(reference, &arguments);
                Some(builder.inst_results(call)[0])
            }
            InstructionKind::Print(register) => {
                let reference = module.declare_func_in_func(print_function, builder.func);
                let value = get_register(&registers, *register)?;
                let kind = match function.register_types[register.0 as usize] {
                    Type::Int => 0,
                    Type::Bool => 1,
                    _ => unreachable!("print type validated"),
                };
                let kind = builder.ins().iconst(types::I8, kind);
                builder
                    .ins()
                    .call(reference, &[context_pointer, value, kind]);
                None
            }
            _ => unreachable!("unsupported instruction rejected"),
        };
        if let (Some(destination), Some(value)) = (instruction.destination, value) {
            registers[destination.0 as usize] = Some(value);
        }
    }
    let Terminator::Return(value) = function.blocks[0].terminator else {
        unreachable!("unsupported terminator rejected")
    };
    let result = value.map_or_else(
        || Ok(builder.ins().iconst(types::I64, 0)),
        |register| get_register(&registers, register),
    )?;
    builder.ins().return_(&[result]);
    builder.finalize();
    module
        .define_function(function_ids[function.id.0 as usize], &mut context)
        .map_err(|error| JitError::Backend(error.to_string()))?;
    module.clear_context(&mut context);
    Ok(())
}

fn get_register(
    registers: &[Option<cranelift_codegen::ir::Value>],
    register: thp_mir::Register,
) -> Result<cranelift_codegen::ir::Value, JitError> {
    registers[register.0 as usize]
        .ok_or_else(|| JitError::Unsupported("a non-dominating register use".to_owned()))
}

fn lower_binary(
    builder: &mut FunctionBuilder<'_>,
    op: BinaryOp,
    left: cranelift_codegen::ir::Value,
    right: cranelift_codegen::ir::Value,
) -> cranelift_codegen::ir::Value {
    let comparison = match op {
        BinaryOp::Equal | BinaryOp::StrictEqual => builder.ins().icmp(IntCC::Equal, left, right),
        BinaryOp::NotEqual => builder.ins().icmp(IntCC::NotEqual, left, right),
        BinaryOp::Less => builder.ins().icmp(IntCC::SignedLessThan, left, right),
        BinaryOp::LessEqual => builder
            .ins()
            .icmp(IntCC::SignedLessThanOrEqual, left, right),
        BinaryOp::Greater => builder.ins().icmp(IntCC::SignedGreaterThan, left, right),
        BinaryOp::GreaterEqual => builder
            .ins()
            .icmp(IntCC::SignedGreaterThanOrEqual, left, right),
        BinaryOp::And => return builder.ins().band(left, right),
        BinaryOp::Or => return builder.ins().bor(left, right),
        _ => unreachable!("unsupported binary operator rejected"),
    };
    builder.ins().uextend(types::I64, comparison)
}

fn decode_result(ty: &Type, raw: i64) -> Value {
    match ty {
        Type::Int => Value::integer(raw),
        Type::Bool => Value::bool(raw != 0),
        Type::Null | Type::Void => Value::NULL,
        _ => unreachable!("unsupported result type rejected"),
    }
}

#[cfg(test)]
mod tests {
    use thp_bytecode::lower as lower_bytecode;
    use thp_diagnostics::SourceFile;
    use thp_hir::lower as lower_hir;
    use thp_mir::lower as lower_mir;
    use thp_syntax::parse;

    use super::{execute, supports};

    fn compile(source: &str) -> thp_bytecode::Program {
        let source = SourceFile::new("jit.thp", source);
        let parsed = parse(&source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let hir = lower_hir(&parsed.program);
        assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);
        lower_bytecode(&lower_mir(&hir.module))
    }

    #[test]
    fn compiles_and_executes_native_function_calls() {
        let mut program =
            compile("<?thp\nfunction answer(): int { return 42; }\n$value: int = answer();");
        program.entry = thp_hir::FunctionId(1);
        let execution = execute(&program).unwrap();
        assert_eq!(execution.compiled_functions, 2);
        assert_eq!(execution.result.as_int(), Some(42));
    }

    #[test]
    fn writes_integer_and_boolean_output() {
        let program = compile("<?thp\necho 42;\necho true;\necho false;");
        assert!(supports(&program));
        let execution = execute(&program).unwrap();
        assert_eq!(execution.output, b"42truefalse");
    }

    #[test]
    fn rejects_heap_operations_for_vm_fallback() {
        let program = compile("<?thp\n$value = \"heap\";");
        assert!(execute(&program).is_err());
    }

    #[test]
    fn rejects_object_dispatch_and_finally_for_vm_fallback() {
        let objects = compile(
            r#"<?thp
class Value {
    public function read(): int { return 1; }
}
echo (new Value())->read() . "";
"#,
        );
        assert!(!supports(&objects));
        assert!(execute(&objects).is_err());

        let cleanup = compile(
            r#"<?thp
try {
    echo "1";
} finally {
    echo "2";
}
"#,
        );
        assert!(!supports(&cleanup));
        assert!(execute(&cleanup).is_err());
    }

    #[test]
    fn string_output_uses_vm_fallback() {
        let program =
            compile("<?thp\nfunction answer(): int { return 42; }\necho answer() . \"\";");
        assert!(!supports(&program));
        assert!(execute(&program).is_err());
        let interpreted = thp_vm::execute(&program, thp_vm::Limits::default()).unwrap();
        assert_eq!(interpreted.output, b"42");
    }

    #[test]
    fn supported_program_matches_the_reference_vm() {
        let mut program = compile("<?thp\nfunction answer(): int { return 42; }");
        program.entry = thp_hir::FunctionId(1);
        let native = execute(&program).unwrap();
        let interpreted = thp_vm::execute(&program, thp_vm::Limits::default()).unwrap();
        assert_eq!(native.output, interpreted.output);
        assert_eq!(native.result, interpreted.result);
    }
}
