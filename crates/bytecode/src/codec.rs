use std::fmt;

use thp_diagnostics::Span;
use thp_hir::{
    Builtin, CalledClass, Callee, ClassId, ConstantValue, FunctionId, LocalId,
    MAX_CONSTANT_NESTING, MethodSlot, NominalKind, ParameterMetadata, PropertyId,
    ReflectionBuiltin, Type, TypeParameter, TypeParameterId,
};
use thp_mir::{BlockId, Constant, Register};
use thp_syntax::{BinaryOp, UnaryOp};

use crate::{
    BYTECODE_SCHEMA_VERSION, Block, CatchHandler, Class, DynamicArgument, ExceptionHandler,
    Function, Instruction, InstructionKind, Method, NominalType, Program, Property, SourceInfo,
    Terminator, verify,
};

const MAGIC: &[u8; 8] = b"THPBC\0\0\0";
const NONE: u32 = u32::MAX;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodeError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid THP bytecode at byte {}: {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for DecodeError {}

pub fn encode(program: &Program) -> Vec<u8> {
    let mut encoder = Encoder {
        bytes: Vec::with_capacity(program.instruction_count() * 16),
    };
    encoder.bytes(MAGIC);
    encoder.u16(program.schema_version);
    encoder.u32(program.entry.0);
    encoder.len(program.sources.len());
    for source in &program.sources {
        encoder.string(&source.path);
        encoder.len(source.line_starts.len());
        for start in &source.line_starts {
            encoder.u32(*start);
        }
    }
    encoder.len(program.classes.len());
    for class in &program.classes {
        encoder.class(class);
    }
    encoder.len(program.functions.len());
    for function in &program.functions {
        encoder.function(function);
    }
    encoder.bytes
}

/// Decodes and verifies a bytecode artifact.
///
/// # Errors
///
/// Returns an offset-bearing error for malformed, unsupported, truncated, or
/// statically invalid bytecode.
pub fn decode(bytes: &[u8]) -> Result<Program, DecodeError> {
    let mut decoder = Decoder { bytes, offset: 0 };
    let magic = decoder.take(MAGIC.len())?;
    if magic != MAGIC {
        return Err(Decoder::error_at(0, "invalid bytecode magic"));
    }
    let schema_version = decoder.u16()?;
    if schema_version != BYTECODE_SCHEMA_VERSION {
        return Err(decoder.error(format!(
            "unsupported schema {schema_version}, expected {BYTECODE_SCHEMA_VERSION}"
        )));
    }
    let entry = FunctionId(decoder.u32()?);
    let sources = decoder.vector(|decoder| {
        let path = decoder.string()?;
        let line_starts = decoder.vector(Decoder::u32)?;
        Ok(SourceInfo { path, line_starts })
    })?;
    let classes = decoder.vector(Decoder::class)?;
    let count = decoder.len()?;
    let mut functions = Vec::with_capacity(count);
    for _ in 0..count {
        functions.push(decoder.function()?);
    }
    if decoder.offset != bytes.len() {
        return Err(decoder.error("trailing bytes after bytecode program"));
    }
    let program = Program {
        schema_version,
        sources,
        functions,
        classes,
        entry,
    };
    verify(&program).map_err(|error| Decoder::error_at(0, error.to_string()))?;
    Ok(program)
}

struct Encoder {
    bytes: Vec<u8>,
}

impl Encoder {
    fn bytes(&mut self, value: &[u8]) {
        self.bytes.extend_from_slice(value);
    }

    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u16(&mut self, value: u16) {
        self.bytes(value.to_le_bytes().as_slice());
    }

    fn u32(&mut self, value: u32) {
        self.bytes(value.to_le_bytes().as_slice());
    }

    fn u64(&mut self, value: u64) {
        self.bytes(value.to_le_bytes().as_slice());
    }

    fn len(&mut self, value: usize) {
        self.u32(u32::try_from(value).expect("bytecode collection is limited to u32::MAX items"));
    }

    fn string(&mut self, value: &str) {
        self.len(value.len());
        self.bytes(value.as_bytes());
    }

    fn blob(&mut self, value: &[u8]) {
        self.len(value.len());
        self.bytes(value);
    }

    fn span(&mut self, span: Span) {
        self.u32(span.start);
        self.u32(span.end);
        self.u32(span.source.map_or(NONE, |source| source.0));
    }

    fn ty(&mut self, ty: &Type) {
        match ty {
            Type::Int => self.u8(0),
            Type::Float => self.u8(1),
            Type::Bool => self.u8(2),
            Type::String => self.u8(3),
            Type::Null => self.u8(4),
            Type::Void => self.u8(5),
            Type::Never => self.u8(6),
            Type::Mixed => self.u8(7),
            Type::Vector(element) => {
                self.u8(8);
                self.ty(element);
            }
            Type::Map(key, value) => {
                self.u8(9);
                self.ty(key);
                self.ty(value);
            }
            Type::Union(members) => {
                self.u8(10);
                self.len(members.len());
                for member in members {
                    self.ty(member);
                }
            }
            Type::Object(name) => {
                self.u8(11);
                self.string(name);
            }
            Type::Nominal { name, arguments } => {
                self.u8(12);
                self.string(name);
                self.len(arguments.len());
                for argument in arguments {
                    self.ty(argument);
                }
            }
            Type::Parameter { id, name } => {
                self.u8(13);
                self.u32(id.owner.0);
                self.u32(id.index);
                self.string(name);
            }
            Type::Callable(parameters, result) => {
                self.u8(14);
                self.len(parameters.len());
                for parameter in parameters {
                    self.ty(parameter);
                }
                self.ty(result);
            }
        }
    }

    fn class(&mut self, class: &Class) {
        self.u32(class.id.0);
        self.string(&class.name);
        self.u8(match class.kind {
            NominalKind::Class => 0,
            NominalKind::Interface => 1,
            NominalKind::Trait => 2,
        });
        self.u8(u8::from(class.abstract_class));
        self.u8(u8::from(class.final_class));
        self.string(&class.module_name);
        self.u8(u8::from(class.native));
        self.len(class.type_parameters.len());
        for parameter in &class.type_parameters {
            self.u32(parameter.id.owner.0);
            self.u32(parameter.id.index);
            self.string(&parameter.name);
            match &parameter.bound {
                Some(bound) => {
                    self.u8(1);
                    self.ty(bound);
                }
                None => self.u8(0),
            }
            self.span(parameter.span);
        }
        self.len(class.properties.len());
        for property in &class.properties {
            self.property(property);
        }
        self.len(class.declared_properties.len());
        for property in &class.declared_properties {
            self.property(property);
        }
        self.len(class.methods.len());
        for method in &class.methods {
            self.method(method);
        }
        self.len(class.declared_methods.len());
        for method in &class.declared_methods {
            self.method(method);
        }
        self.len(class.dispatch.len());
        for callee in &class.dispatch {
            self.optional_callee(*callee);
        }
        self.len(class.interfaces.len());
        for interface in &class.interfaces {
            self.u32(interface.0);
        }
        self.len(class.interface_types.len());
        for interface in &class.interface_types {
            self.u32(interface.class.0);
            self.len(interface.arguments.len());
            for argument in &interface.arguments {
                self.ty(argument);
            }
        }
        self.u32(class.parent.map_or(NONE, |parent| parent.0));
        match &class.parent_type {
            Some(parent) => {
                self.u8(1);
                self.u32(parent.class.0);
                self.len(parent.arguments.len());
                for argument in &parent.arguments {
                    self.ty(argument);
                }
            }
            None => self.u8(0),
        }
        self.len(class.traits.len());
        for trait_id in &class.traits {
            self.u32(trait_id.0);
        }
    }

    fn property(&mut self, property: &Property) {
        self.u32(property.id.0);
        self.string(&property.name);
        self.ty(&property.ty);
        self.visibility(property.visibility);
        self.u32(property.declaring_class.0);
        self.optional_constant_value(property.default.as_ref());
        self.u32(property.origin_trait.map_or(NONE, |origin| origin.0));
    }

    fn method(&mut self, method: &Method) {
        self.string(&method.name);
        self.u32(method.slot.0);
        self.optional_callee(method.callee);
        self.visibility(method.visibility);
        self.u32(method.declaring_class.0);
        self.u8(u8::from(method.static_method));
        self.u8(u8::from(method.abstract_method));
        self.u8(u8::from(method.final_method));
        self.len(method.parameter_types.len());
        for ty in &method.parameter_types {
            self.ty(ty);
        }
        self.len(method.parameters.len());
        for parameter in &method.parameters {
            self.parameter_metadata(parameter);
        }
        self.ty(&method.return_type);
        self.u32(method.origin_trait.map_or(NONE, |origin| origin.0));
        self.string(&method.origin_name);
    }

    fn parameter_metadata(&mut self, parameter: &ParameterMetadata) {
        self.string(&parameter.name);
        self.ty(&parameter.ty);
        self.optional_constant_value(parameter.default.as_ref());
        self.u8(u8::from(parameter.variadic));
    }

    fn optional_constant_value(&mut self, value: Option<&ConstantValue>) {
        match value {
            Some(value) => {
                self.u8(1);
                self.constant_value(value);
            }
            None => self.u8(0),
        }
    }

    fn constant_value(&mut self, value: &ConstantValue) {
        match value {
            ConstantValue::Int(value) => {
                self.u8(0);
                self.u64(u64::from_ne_bytes(value.to_ne_bytes()));
            }
            ConstantValue::Float(value) => {
                self.u8(1);
                self.u64(value.to_bits());
            }
            ConstantValue::Bool(value) => {
                self.u8(2);
                self.u8(u8::from(*value));
            }
            ConstantValue::Null => self.u8(3),
            ConstantValue::String(value) => {
                self.u8(4);
                self.blob(value);
            }
            ConstantValue::Vector(values) => {
                self.u8(5);
                self.len(values.len());
                for value in values {
                    self.constant_value(value);
                }
            }
            ConstantValue::Map(entries) => {
                self.u8(6);
                self.len(entries.len());
                for (key, value) in entries {
                    self.constant_value(key);
                    self.constant_value(value);
                }
            }
        }
    }

    fn function(&mut self, function: &Function) {
        self.u32(function.id.0);
        self.string(&function.name);
        self.string(&function.module_name);
        self.len(function.parameters.len());
        for parameter in &function.parameters {
            self.u32(parameter.0);
        }
        self.len(function.parameter_metadata.len());
        for parameter in &function.parameter_metadata {
            self.parameter_metadata(parameter);
        }
        self.len(function.local_types.len());
        for ty in &function.local_types {
            self.ty(ty);
        }
        self.ty(&function.return_type);
        self.u8(u8::from(function.generator));
        self.u32(function.owner.map_or(NONE, |owner| owner.0));
        self.u8(u8::from(function.static_method));
        self.len(function.register_types.len());
        for ty in &function.register_types {
            self.ty(ty);
        }
        self.u32(function.entry.0);
        self.span(function.span);
        self.len(function.blocks.len());
        for block in &function.blocks {
            self.block(block);
        }
        self.len(function.exception_handlers.len());
        for handler in &function.exception_handlers {
            self.exception_handler(handler);
        }
    }

    fn exception_handler(&mut self, handler: &ExceptionHandler) {
        self.len(handler.protected_blocks.len());
        for block in &handler.protected_blocks {
            self.u32(block.0);
        }
        self.len(handler.catches.len());
        for clause in &handler.catches {
            self.u32(clause.class.map_or(NONE, |class| class.0));
            self.u32(clause.local.0);
            self.u32(clause.target.0);
        }
    }

    fn block(&mut self, block: &Block) {
        self.u32(block.id.0);
        self.len(block.instructions.len());
        for instruction in &block.instructions {
            self.instruction(instruction);
        }
        self.terminator(&block.terminator);
    }

    fn instruction(&mut self, instruction: &Instruction) {
        self.u32(instruction.destination.map_or(NONE, |register| register.0));
        match &instruction.ty {
            Some(ty) => {
                self.u8(1);
                self.ty(ty);
            }
            None => self.u8(0),
        }
        self.span(instruction.span);
        match &instruction.kind {
            InstructionKind::Constant(constant) => {
                self.u8(0);
                self.constant(constant);
            }
            InstructionKind::LoadLocal(local) => {
                self.u8(1);
                self.u32(local.0);
            }
            InstructionKind::StoreLocal { local, value } => {
                self.u8(2);
                self.u32(local.0);
                self.u32(value.0);
            }
            InstructionKind::Unary { op, operand } => {
                self.u8(3);
                self.unary(*op);
                self.u32(operand.0);
            }
            InstructionKind::Binary { op, left, right } => {
                self.u8(4);
                self.binary(*op);
                self.u32(left.0);
                self.u32(right.0);
            }
            InstructionKind::IsNull(register) => {
                self.u8(5);
                self.u32(register.0);
            }
            InstructionKind::Vector(registers) => {
                self.u8(6);
                self.len(registers.len());
                for register in registers {
                    self.u32(register.0);
                }
            }
            InstructionKind::Map(entries) => {
                self.u8(7);
                self.len(entries.len());
                for (key, value) in entries {
                    self.u32(key.0);
                    self.u32(value.0);
                }
            }
            InstructionKind::Index { collection, index } => {
                self.u8(8);
                self.u32(collection.0);
                self.u32(index.0);
            }
            InstructionKind::Call { callee, arguments } => {
                self.u8(9);
                self.callee(*callee);
                self.len(arguments.len());
                for argument in arguments {
                    self.u32(argument.0);
                }
            }
            InstructionKind::Phi(inputs) => {
                self.u8(10);
                self.len(inputs.len());
                for (block, register) in inputs {
                    self.u32(block.0);
                    self.u32(register.0);
                }
            }
            InstructionKind::Print(register) => {
                self.u8(11);
                self.u32(register.0);
            }
            InstructionKind::NewObject(class) => {
                self.u8(12);
                self.u32(class.0);
            }
            InstructionKind::GetProperty { object, property } => {
                self.u8(13);
                self.u32(object.0);
                self.u32(property.0);
            }
            InstructionKind::SetProperty {
                object,
                property,
                value,
            } => {
                self.u8(14);
                self.u32(object.0);
                self.u32(property.0);
                self.u32(value.0);
            }
            InstructionKind::InstanceOf { value, class } => {
                self.u8(15);
                self.u32(value.0);
                self.u32(class.0);
            }
            InstructionKind::AddSuppressed {
                primary,
                suppressed,
            } => {
                self.u8(16);
                self.u32(primary.0);
                self.u32(suppressed.0);
            }
            InstructionKind::CollectionLen(collection) => {
                self.u8(17);
                self.u32(collection.0);
            }
            InstructionKind::CollectionKeyAt { collection, offset } => {
                self.u8(18);
                self.u32(collection.0);
                self.u32(offset.0);
            }
            InstructionKind::CollectionValueAt { collection, offset } => {
                self.u8(19);
                self.u32(collection.0);
                self.u32(offset.0);
            }
            InstructionKind::SetIndex {
                collection,
                index,
                value,
            } => {
                self.u8(20);
                self.u32(collection.0);
                self.u32(index.0);
                self.u32(value.0);
            }
            InstructionKind::RaiseUnhandledMatch(value) => {
                self.u8(21);
                self.u32(value.0);
            }
            InstructionKind::DirectMethod {
                callee,
                arguments,
                called_class,
            } => {
                self.u8(22);
                self.callee(*callee);
                self.len(arguments.len());
                for argument in arguments {
                    self.u32(argument.0);
                }
                match called_class {
                    CalledClass::Explicit(class) => {
                        self.u8(0);
                        self.u32(class.0);
                    }
                    CalledClass::Forwarded => self.u8(1),
                    CalledClass::Receiver => self.u8(2),
                }
            }
            InstructionKind::VirtualMethod {
                receiver,
                slot,
                arguments,
            } => {
                self.u8(23);
                self.u32(receiver.0);
                self.u32(slot.0);
                self.len(arguments.len());
                for argument in arguments {
                    self.u32(argument.0);
                }
            }
            InstructionKind::LateStaticMethod {
                receiver,
                slot,
                arguments,
            } => {
                self.u8(24);
                self.u32(receiver.map_or(NONE, |receiver| receiver.0));
                self.u32(slot.0);
                self.len(arguments.len());
                for argument in arguments {
                    self.u32(argument.0);
                }
            }
            InstructionKind::ChainPrevious {
                replacement,
                previous,
            } => {
                self.u8(25);
                self.u32(replacement.0);
                self.u32(previous.0);
            }
            InstructionKind::InitializeProperty {
                object,
                property,
                value,
            } => {
                self.u8(26);
                self.u32(object.0);
                self.u32(property.0);
                self.u32(value.0);
            }
            InstructionKind::NewDynamic {
                target,
                type_arguments,
                arguments,
            } => {
                self.u8(27);
                self.u32(target.0);
                self.len(type_arguments.len());
                for ty in type_arguments {
                    self.ty(ty);
                }
                self.len(arguments.len());
                for argument in arguments {
                    if let Some(name) = &argument.name {
                        self.u8(1);
                        self.string(name);
                    } else {
                        self.u8(0);
                    }
                    self.u32(argument.value.0);
                    self.span(argument.span);
                }
            }
            InstructionKind::CheckedNarrow { value, narrowed } => {
                self.u8(28);
                self.u32(value.0);
                self.ty(narrowed);
            }
            InstructionKind::Closure { function, captures } => {
                self.u8(29);
                self.u32(function.0);
                self.len(captures.len());
                for capture in captures {
                    self.u32(capture.0);
                }
            }
            InstructionKind::CallValue { callee, arguments } => {
                self.u8(30);
                self.u32(callee.0);
                self.len(arguments.len());
                for argument in arguments {
                    self.u32(argument.0);
                }
            }
        }
    }

    fn constant(&mut self, constant: &Constant) {
        match constant {
            Constant::Integer(value) => {
                self.u8(0);
                self.u64(u64::from_ne_bytes(value.to_ne_bytes()));
            }
            Constant::Float(value) => {
                self.u8(1);
                self.u64(value.to_bits());
            }
            Constant::Bool(value) => {
                self.u8(2);
                self.u8(u8::from(*value));
            }
            Constant::Null => self.u8(3),
            Constant::String(value) => {
                self.u8(4);
                self.blob(value);
            }
        }
    }

    fn unary(&mut self, op: UnaryOp) {
        self.u8(match op {
            UnaryOp::Negate => 0,
            UnaryOp::Not => 1,
        });
    }

    fn binary(&mut self, op: BinaryOp) {
        self.u8(match op {
            BinaryOp::Add => 0,
            BinaryOp::Subtract => 1,
            BinaryOp::Multiply => 2,
            BinaryOp::Divide => 3,
            BinaryOp::Remainder => 4,
            BinaryOp::Concatenate => 5,
            BinaryOp::Equal => 6,
            BinaryOp::StrictEqual => 7,
            BinaryOp::NotEqual => 8,
            BinaryOp::Less => 9,
            BinaryOp::LessEqual => 10,
            BinaryOp::Greater => 11,
            BinaryOp::GreaterEqual => 12,
            BinaryOp::And => 13,
            BinaryOp::Or => 14,
            BinaryOp::Coalesce => 15,
        });
    }

    fn callee(&mut self, callee: Callee) {
        match callee {
            Callee::Function(function) => {
                self.u8(0);
                self.u32(function.0);
            }
            Callee::Builtin(Builtin::Count) => self.u8(1),
            Callee::Builtin(Builtin::VarDump) => self.u8(2),
            Callee::Builtin(Builtin::MemoryStreamOpen) => self.u8(3),
            Callee::Builtin(Builtin::TempStreamOpen) => self.u8(4),
            Callee::Builtin(Builtin::StreamsOpen) => self.u8(5),
            Callee::Builtin(Builtin::FilesOpenRead) => self.u8(6),
            Callee::Builtin(Builtin::StreamTell) => self.u8(7),
            Callee::Builtin(Builtin::StreamRead) => self.u8(8),
            Callee::Builtin(Builtin::StreamReadAll) => self.u8(9),
            Callee::Builtin(Builtin::StreamEof) => self.u8(10),
            Callee::Builtin(Builtin::StreamSeek) => self.u8(11),
            Callee::Builtin(Builtin::StreamWriteAll) => self.u8(12),
            Callee::Builtin(Builtin::StreamClose) => self.u8(13),
            Callee::Builtin(Builtin::StreamIsClosed) => self.u8(14),
            Callee::Builtin(Builtin::ExceptionNew) => self.u8(15),
            Callee::Builtin(Builtin::ExceptionGetMessage) => self.u8(16),
            Callee::Builtin(Builtin::ExceptionGetTarget) => self.u8(17),
            Callee::Builtin(Builtin::ExceptionGetSystemCode) => self.u8(18),
            Callee::Builtin(Builtin::ExceptionGetSuppressed) => self.u8(19),
            Callee::Builtin(Builtin::ExceptionConstruct) => self.u8(20),
            Callee::Builtin(Builtin::ExceptionGetCode) => self.u8(21),
            Callee::Builtin(Builtin::ExceptionGetPrevious) => self.u8(22),
            Callee::Builtin(Builtin::Reflection(operation)) => {
                self.u8(23);
                self.u8(operation as u8);
            }
            Callee::Builtin(Builtin::IsString) => self.u8(24),
            Callee::Builtin(Builtin::IsInt) => self.u8(25),
            Callee::Builtin(Builtin::IsFloat) => self.u8(26),
            Callee::Builtin(Builtin::IsNull) => self.u8(27),
            Callee::Builtin(Builtin::IsNumeric) => self.u8(28),
            Callee::Builtin(Builtin::IsVector) => self.u8(29),
            Callee::Builtin(Builtin::IsMap) => self.u8(30),
            Callee::Builtin(Builtin::IteratorFromCollection) => self.u8(31),
            Callee::Builtin(Builtin::IteratorConstruct) => self.u8(32),
            Callee::Builtin(Builtin::IteratorRewind) => self.u8(33),
            Callee::Builtin(Builtin::IteratorValid) => self.u8(34),
            Callee::Builtin(Builtin::IteratorKey) => self.u8(35),
            Callee::Builtin(Builtin::IteratorValue) => self.u8(36),
            Callee::Builtin(Builtin::IteratorAdvance) => self.u8(37),
            Callee::Builtin(Builtin::IteratorGetInner) => self.u8(38),
            Callee::Builtin(Builtin::IteratorCount) => self.u8(39),
            Callee::Builtin(Builtin::IteratorToVector) => self.u8(40),
            Callee::Builtin(Builtin::IteratorToMap) => self.u8(41),
            Callee::Builtin(Builtin::VectorMap) => self.u8(42),
            Callee::Builtin(Builtin::VectorFilter) => self.u8(43),
            Callee::Builtin(Builtin::VectorSlice) => self.u8(44),
            Callee::Builtin(Builtin::VectorConcat) => self.u8(45),
            Callee::Builtin(Builtin::MapTransform) => self.u8(46),
            Callee::Builtin(Builtin::MapFilter) => self.u8(47),
            Callee::Builtin(Builtin::MapMerge) => self.u8(48),
            Callee::Builtin(Builtin::IteratorApply) => self.u8(49),
            Callee::Builtin(Builtin::CallbackFilterAccept) => self.u8(50),
            Callee::Builtin(Builtin::CallbackFilterConstruct) => self.u8(51),
            Callee::Builtin(Builtin::GeneratorGetReturn) => self.u8(52),
            Callee::Builtin(Builtin::GeneratorClose) => self.u8(53),
            Callee::Builtin(Builtin::OptionSome) => self.u8(54),
            Callee::Builtin(Builtin::OptionNone) => self.u8(55),
            Callee::Builtin(Builtin::OptionIsSome) => self.u8(56),
            Callee::Builtin(Builtin::OptionIsNone) => self.u8(57),
            Callee::Builtin(Builtin::OptionGet) => self.u8(58),
            Callee::Builtin(Builtin::Serialize) => self.u8(59),
            Callee::Builtin(Builtin::Unserialize) => self.u8(60),
            Callee::Builtin(Builtin::TraceLineConstruct) => self.u8(61),
            Callee::Builtin(Builtin::ExceptionGetFile) => self.u8(62),
            Callee::Builtin(Builtin::ExceptionGetLine) => self.u8(63),
            Callee::Builtin(Builtin::ExceptionGetTrace) => self.u8(64),
            Callee::Builtin(Builtin::ExceptionGetTraceAsString) => self.u8(65),
            Callee::Builtin(Builtin::ExceptionToString) => self.u8(66),
            Callee::Builtin(Builtin::LimitSeek) => self.u8(67),
            Callee::Builtin(Builtin::LimitGetPosition) => self.u8(68),
            Callee::Builtin(Builtin::AppendAdd) => self.u8(69),
            Callee::Builtin(Builtin::AppendGetIndex) => self.u8(70),
            Callee::Builtin(Builtin::AppendGetIterator) => self.u8(71),
            Callee::Builtin(Builtin::CachingHasNext) => self.u8(72),
            Callee::Builtin(Builtin::CachingCount) => self.u8(73),
            Callee::Builtin(Builtin::CachingGetCache) => self.u8(74),
            Callee::Builtin(Builtin::CachingGetFlags) => self.u8(75),
            Callee::Builtin(Builtin::CachingSetFlags) => self.u8(76),
            Callee::Builtin(Builtin::CachingOffsetExists) => self.u8(77),
            Callee::Builtin(Builtin::CachingOffsetGet) => self.u8(78),
            Callee::Builtin(Builtin::CachingOffsetSet) => self.u8(79),
            Callee::Builtin(Builtin::CachingOffsetUnset) => self.u8(80),
            Callee::Builtin(Builtin::CachingToString) => self.u8(81),
            Callee::Builtin(Builtin::RecursiveEntryConstruct) => self.u8(82),
            Callee::Builtin(Builtin::RecursiveEntryValue) => self.u8(83),
            Callee::Builtin(Builtin::RecursiveEntryChildren) => self.u8(84),
            Callee::Builtin(Builtin::RecursiveWalkConstruct) => self.u8(85),
            Callee::Builtin(Builtin::RecursiveWalkSetMaxDepth) => self.u8(86),
            Callee::Builtin(Builtin::RecursiveWalkGetMaxDepth) => self.u8(87),
            Callee::Builtin(Builtin::RecursiveWalkGetDepth) => self.u8(88),
            Callee::Builtin(Builtin::RecursiveWalkGetSubIterator) => self.u8(89),
            Callee::Builtin(Builtin::RecursiveWalkGetRecursiveIterator) => self.u8(90),
            Callee::Builtin(Builtin::RecursiveWalkHook) => self.u8(91),
            Callee::Builtin(Builtin::FixedConstruct) => self.u8(92),
            Callee::Builtin(Builtin::FixedSetSize) => self.u8(93),
            Callee::Builtin(Builtin::FixedGetSize) => self.u8(94),
            Callee::Builtin(Builtin::FixedOffsetExists) => self.u8(95),
            Callee::Builtin(Builtin::FixedOffsetGet) => self.u8(96),
            Callee::Builtin(Builtin::FixedOffsetSet) => self.u8(97),
            Callee::Builtin(Builtin::FixedOffsetUnset) => self.u8(98),
            Callee::Builtin(Builtin::FixedGetIterator) => self.u8(99),
            Callee::Builtin(Builtin::ListConstruct) => self.u8(100),
            Callee::Builtin(Builtin::ListPush) => self.u8(101),
            Callee::Builtin(Builtin::ListPop) => self.u8(102),
            Callee::Builtin(Builtin::ListUnshift) => self.u8(103),
            Callee::Builtin(Builtin::ListShift) => self.u8(104),
            Callee::Builtin(Builtin::ListTop) => self.u8(105),
            Callee::Builtin(Builtin::ListBottom) => self.u8(106),
            Callee::Builtin(Builtin::ListCount) => self.u8(107),
            Callee::Builtin(Builtin::ListIsEmpty) => self.u8(108),
            Callee::Builtin(Builtin::ListGetIterator) => self.u8(109),
            Callee::Builtin(Builtin::ListToVector) => self.u8(110),
            Callee::Builtin(Builtin::HeapConstruct) => self.u8(111),
            Callee::Builtin(Builtin::HeapInsert) => self.u8(112),
            Callee::Builtin(Builtin::HeapTop) => self.u8(113),
            Callee::Builtin(Builtin::HeapExtract) => self.u8(114),
            Callee::Builtin(Builtin::HeapCount) => self.u8(115),
            Callee::Builtin(Builtin::HeapIsEmpty) => self.u8(116),
            Callee::Builtin(Builtin::HeapGetIterator) => self.u8(117),
            Callee::Builtin(Builtin::TypedMapConstruct) => self.u8(118),
            Callee::Builtin(Builtin::TypedMapSet) => self.u8(119),
            Callee::Builtin(Builtin::TypedMapGet) => self.u8(120),
            Callee::Builtin(Builtin::TypedMapContains) => self.u8(121),
            Callee::Builtin(Builtin::TypedMapRemove) => self.u8(122),
            Callee::Builtin(Builtin::TypedMapCount) => self.u8(123),
            Callee::Builtin(Builtin::TypedMapToMap) => self.u8(124),
            Callee::Builtin(Builtin::TypedMapGetIterator) => self.u8(125),
            Callee::Builtin(Builtin::ObjectStorageConstruct) => self.u8(126),
            Callee::Builtin(Builtin::ObjectStorageAttach) => self.u8(127),
            Callee::Builtin(Builtin::ObjectStorageGet) => self.u8(128),
            Callee::Builtin(Builtin::ObjectStorageContains) => self.u8(129),
            Callee::Builtin(Builtin::ObjectStorageDetach) => self.u8(130),
            Callee::Builtin(Builtin::ObjectStorageCount) => self.u8(131),
            Callee::Builtin(Builtin::ObjectStorageGetIterator) => self.u8(132),
            Callee::Builtin(Builtin::ParentAccept) => self.u8(133),
            Callee::Builtin(Builtin::RecursiveCallbackConstruct) => self.u8(134),
            Callee::Builtin(Builtin::RecursiveCallbackAccept) => self.u8(135),
        }
    }

    fn optional_callee(&mut self, callee: Option<Callee>) {
        match callee {
            Some(callee) => {
                self.u8(1);
                self.callee(callee);
            }
            None => self.u8(0),
        }
    }

    fn visibility(&mut self, visibility: thp_syntax::Visibility) {
        self.u8(match visibility {
            thp_syntax::Visibility::Public => 0,
            thp_syntax::Visibility::Protected => 1,
            thp_syntax::Visibility::Private => 2,
        });
    }

    fn terminator(&mut self, terminator: &Terminator) {
        match terminator {
            Terminator::Jump(target) => {
                self.u8(0);
                self.u32(target.0);
            }
            Terminator::Branch {
                condition,
                then_block,
                else_block,
            } => {
                self.u8(1);
                self.u32(condition.0);
                self.u32(then_block.0);
                self.u32(else_block.0);
            }
            Terminator::Return(value) => {
                self.u8(2);
                self.u32(value.map_or(NONE, |register| register.0));
            }
            Terminator::Unreachable => self.u8(3),
            Terminator::Throw(value) => {
                self.u8(4);
                self.u32(value.0);
            }
            Terminator::Yield { key, value, resume } => {
                self.u8(5);
                self.u32(key.map_or(NONE, |register| register.0));
                self.u32(value.0);
                self.u32(resume.0);
            }
        }
    }
}

struct Decoder<'bytes> {
    bytes: &'bytes [u8],
    offset: usize,
}

impl Decoder<'_> {
    fn take(&mut self, count: usize) -> Result<&[u8], DecodeError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| self.error("byte offset overflow"))?;
        let Some(value) = self.bytes.get(self.offset..end) else {
            return Err(self.error("unexpected end of bytecode"));
        };
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("slice length checked"),
        ))
    }

    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("slice length checked"),
        ))
    }

    fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("slice length checked"),
        ))
    }

    fn len(&mut self) -> Result<usize, DecodeError> {
        Ok(self.u32()? as usize)
    }

    fn string(&mut self) -> Result<String, DecodeError> {
        let length = self.len()?;
        let offset = self.offset;
        let bytes = self.take(length)?;
        let Ok(value) = std::str::from_utf8(bytes) else {
            return Err(DecodeError {
                offset,
                message: "bytecode string is not valid UTF-8".to_owned(),
            });
        };
        Ok(value.to_owned())
    }

    fn blob(&mut self) -> Result<Vec<u8>, DecodeError> {
        let length = self.len()?;
        Ok(self.take(length)?.to_vec())
    }

    fn span(&mut self) -> Result<Span, DecodeError> {
        let start = self.u32()?;
        let end = self.u32()?;
        let source = self.u32()?;
        if end < start {
            return Err(self.error("source span ends before it starts"));
        }
        Ok(Span {
            start,
            end,
            source: (source != NONE).then_some(thp_diagnostics::SourceId(source)),
        })
    }

    fn ty(&mut self, depth: usize) -> Result<Type, DecodeError> {
        if depth > 128 {
            return Err(self.error("type nesting exceeds 128 levels"));
        }
        Ok(match self.u8()? {
            0 => Type::Int,
            1 => Type::Float,
            2 => Type::Bool,
            3 => Type::String,
            4 => Type::Null,
            5 => Type::Void,
            6 => Type::Never,
            7 => Type::Mixed,
            8 => Type::Vector(Box::new(self.ty(depth + 1)?)),
            9 => Type::Map(Box::new(self.ty(depth + 1)?), Box::new(self.ty(depth + 1)?)),
            10 => {
                let count = self.len()?;
                if count > 256 {
                    return Err(self.error("union has more than 256 members"));
                }
                let mut members = Vec::with_capacity(count);
                for _ in 0..count {
                    members.push(self.ty(depth + 1)?);
                }
                Type::Union(members)
            }
            11 => Type::Object(self.string()?),
            12 => Type::Nominal {
                name: self.string()?,
                arguments: self.vector(|decoder| decoder.ty(depth + 1))?,
            },
            13 => Type::Parameter {
                id: TypeParameterId {
                    owner: ClassId(self.u32()?),
                    index: self.u32()?,
                },
                name: self.string()?,
            },
            14 => Type::Callable(
                self.vector(|decoder| decoder.ty(depth + 1))?,
                Box::new(self.ty(depth + 1)?),
            ),
            tag => return Err(self.error(format!("unknown type tag {tag}"))),
        })
    }

    fn class(&mut self) -> Result<Class, DecodeError> {
        let id = ClassId(self.u32()?);
        let name = self.string()?;
        let kind = match self.u8()? {
            0 => NominalKind::Class,
            1 => NominalKind::Interface,
            2 => NominalKind::Trait,
            tag => return Err(self.error(format!("unknown nominal kind tag {tag}"))),
        };
        let abstract_class = self.boolean()?;
        let final_class = self.boolean()?;
        let module_name = self.string()?;
        let native = self.boolean()?;
        let type_parameters = self.vector(|decoder| {
            let id = TypeParameterId {
                owner: ClassId(decoder.u32()?),
                index: decoder.u32()?,
            };
            let name = decoder.string()?;
            let bound = if decoder.boolean()? {
                Some(decoder.ty(0)?)
            } else {
                None
            };
            let span = decoder.span()?;
            Ok(TypeParameter {
                id,
                name,
                bound,
                span,
            })
        })?;
        let properties = self.vector(Self::property)?;
        let declared_properties = self.vector(Self::property)?;
        let methods = self.vector(Self::method)?;
        let declared_methods = self.vector(Self::method)?;
        let dispatch = self.vector(Self::optional_callee)?;
        let interfaces = self.vector(|decoder| Ok(ClassId(decoder.u32()?)))?;
        let interface_types = self.vector(|decoder| {
            Ok(NominalType {
                class: ClassId(decoder.u32()?),
                arguments: decoder.vector(|decoder| decoder.ty(0))?,
            })
        })?;
        let parent = match self.u32()? {
            NONE => None,
            parent => Some(ClassId(parent)),
        };
        let parent_type = if self.boolean()? {
            Some(NominalType {
                class: ClassId(self.u32()?),
                arguments: self.vector(|decoder| decoder.ty(0))?,
            })
        } else {
            None
        };
        let traits = self.vector(|decoder| Ok(ClassId(decoder.u32()?)))?;
        Ok(Class {
            id,
            name,
            kind,
            abstract_class,
            final_class,
            module_name,
            native,
            type_parameters,
            properties,
            declared_properties,
            methods,
            declared_methods,
            dispatch,
            interfaces,
            interface_types,
            parent,
            parent_type,
            traits,
        })
    }

    fn property(&mut self) -> Result<Property, DecodeError> {
        Ok(Property {
            id: PropertyId(self.u32()?),
            name: self.string()?,
            ty: self.ty(0)?,
            visibility: self.visibility()?,
            declaring_class: ClassId(self.u32()?),
            default: self.optional_constant_value(0)?,
            origin_trait: match self.u32()? {
                NONE => None,
                origin => Some(ClassId(origin)),
            },
        })
    }

    fn method(&mut self) -> Result<Method, DecodeError> {
        Ok(Method {
            name: self.string()?,
            slot: MethodSlot(self.u32()?),
            callee: self.optional_callee()?,
            visibility: self.visibility()?,
            declaring_class: ClassId(self.u32()?),
            static_method: self.boolean()?,
            abstract_method: self.boolean()?,
            final_method: self.boolean()?,
            parameter_types: self.vector(|decoder| decoder.ty(0))?,
            parameters: self.vector(Self::parameter_metadata)?,
            return_type: self.ty(0)?,
            origin_trait: match self.u32()? {
                NONE => None,
                origin => Some(ClassId(origin)),
            },
            origin_name: self.string()?,
        })
    }

    fn parameter_metadata(&mut self) -> Result<ParameterMetadata, DecodeError> {
        Ok(ParameterMetadata {
            name: self.string()?,
            ty: self.ty(0)?,
            default: self.optional_constant_value(0)?,
            variadic: self.boolean()?,
        })
    }

    fn optional_constant_value(
        &mut self,
        depth: usize,
    ) -> Result<Option<ConstantValue>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => self.constant_value(depth).map(Some),
            tag => Err(self.error(format!("invalid optional constant tag {tag}"))),
        }
    }

    fn constant_value(&mut self, depth: usize) -> Result<ConstantValue, DecodeError> {
        if depth > MAX_CONSTANT_NESTING {
            return Err(self.error(format!(
                "constant nesting exceeds {MAX_CONSTANT_NESTING} levels"
            )));
        }
        Ok(match self.u8()? {
            0 => ConstantValue::Int(i64::from_ne_bytes(self.u64()?.to_ne_bytes())),
            1 => ConstantValue::Float(f64::from_bits(self.u64()?)),
            2 => ConstantValue::Bool(self.boolean()?),
            3 => ConstantValue::Null,
            4 => ConstantValue::String(self.blob()?),
            5 => ConstantValue::Vector(self.vector(|decoder| decoder.constant_value(depth + 1))?),
            6 => ConstantValue::Map(self.vector(|decoder| {
                Ok((
                    decoder.constant_value(depth + 1)?,
                    decoder.constant_value(depth + 1)?,
                ))
            })?),
            tag => return Err(self.error(format!("invalid constant metadata tag {tag}"))),
        })
    }

    fn function(&mut self) -> Result<Function, DecodeError> {
        let id = FunctionId(self.u32()?);
        let name = self.string()?;
        let module_name = self.string()?;
        let parameters = self.vector(|decoder| Ok(LocalId(decoder.u32()?)))?;
        let parameter_metadata = self.vector(Self::parameter_metadata)?;
        let local_types = self.vector(|decoder| decoder.ty(0))?;
        let return_type = self.ty(0)?;
        let generator = self.boolean()?;
        let owner = match self.u32()? {
            NONE => None,
            owner => Some(ClassId(owner)),
        };
        let static_method = self.boolean()?;
        let register_types = self.vector(|decoder| decoder.ty(0))?;
        let entry = BlockId(self.u32()?);
        let span = self.span()?;
        let blocks = self.vector(Self::block)?;
        let exception_handlers = self.vector(Self::exception_handler)?;
        Ok(Function {
            id,
            name,
            module_name,
            parameters,
            parameter_metadata,
            local_types,
            return_type,
            generator,
            owner,
            static_method,
            register_types,
            blocks,
            exception_handlers,
            entry,
            span,
        })
    }

    fn exception_handler(&mut self) -> Result<ExceptionHandler, DecodeError> {
        Ok(ExceptionHandler {
            protected_blocks: self.vector(|decoder| Ok(BlockId(decoder.u32()?)))?,
            catches: self.vector(|decoder| {
                let class = match decoder.u32()? {
                    NONE => None,
                    class => Some(ClassId(class)),
                };
                Ok(CatchHandler {
                    class,
                    local: LocalId(decoder.u32()?),
                    target: BlockId(decoder.u32()?),
                })
            })?,
        })
    }

    fn block(&mut self) -> Result<Block, DecodeError> {
        let id = BlockId(self.u32()?);
        let instructions = self.vector(Self::instruction)?;
        let terminator = self.terminator()?;
        Ok(Block {
            id,
            instructions,
            terminator,
        })
    }

    fn instruction(&mut self) -> Result<Instruction, DecodeError> {
        let destination = match self.u32()? {
            NONE => None,
            register => Some(Register(register)),
        };
        let ty = match self.u8()? {
            0 => None,
            1 => Some(self.ty(0)?),
            tag => return Err(self.error(format!("invalid optional type tag {tag}"))),
        };
        let span = self.span()?;
        let kind = match self.u8()? {
            0 => InstructionKind::Constant(self.constant()?),
            1 => InstructionKind::LoadLocal(LocalId(self.u32()?)),
            2 => InstructionKind::StoreLocal {
                local: LocalId(self.u32()?),
                value: Register(self.u32()?),
            },
            3 => InstructionKind::Unary {
                op: self.unary()?,
                operand: Register(self.u32()?),
            },
            4 => InstructionKind::Binary {
                op: self.binary()?,
                left: Register(self.u32()?),
                right: Register(self.u32()?),
            },
            5 => InstructionKind::IsNull(Register(self.u32()?)),
            6 => InstructionKind::Vector(self.vector(|decoder| Ok(Register(decoder.u32()?)))?),
            7 => InstructionKind::Map(
                self.vector(|decoder| Ok((Register(decoder.u32()?), Register(decoder.u32()?))))?,
            ),
            8 => InstructionKind::Index {
                collection: Register(self.u32()?),
                index: Register(self.u32()?),
            },
            9 => InstructionKind::Call {
                callee: self.callee()?,
                arguments: self.vector(|decoder| Ok(Register(decoder.u32()?)))?,
            },
            10 => InstructionKind::Phi(
                self.vector(|decoder| Ok((BlockId(decoder.u32()?), Register(decoder.u32()?))))?,
            ),
            11 => InstructionKind::Print(Register(self.u32()?)),
            12 => InstructionKind::NewObject(ClassId(self.u32()?)),
            13 => InstructionKind::GetProperty {
                object: Register(self.u32()?),
                property: PropertyId(self.u32()?),
            },
            14 => InstructionKind::SetProperty {
                object: Register(self.u32()?),
                property: PropertyId(self.u32()?),
                value: Register(self.u32()?),
            },
            15 => InstructionKind::InstanceOf {
                value: Register(self.u32()?),
                class: ClassId(self.u32()?),
            },
            16 => InstructionKind::AddSuppressed {
                primary: Register(self.u32()?),
                suppressed: Register(self.u32()?),
            },
            17 => InstructionKind::CollectionLen(Register(self.u32()?)),
            18 => InstructionKind::CollectionKeyAt {
                collection: Register(self.u32()?),
                offset: Register(self.u32()?),
            },
            19 => InstructionKind::CollectionValueAt {
                collection: Register(self.u32()?),
                offset: Register(self.u32()?),
            },
            20 => InstructionKind::SetIndex {
                collection: Register(self.u32()?),
                index: Register(self.u32()?),
                value: Register(self.u32()?),
            },
            21 => InstructionKind::RaiseUnhandledMatch(Register(self.u32()?)),
            22 => {
                let callee = self.callee()?;
                let arguments = self.vector(|decoder| Ok(Register(decoder.u32()?)))?;
                let called_class = match self.u8()? {
                    0 => CalledClass::Explicit(ClassId(self.u32()?)),
                    1 => CalledClass::Forwarded,
                    2 => CalledClass::Receiver,
                    tag => return Err(self.error(format!("unknown called-class tag {tag}"))),
                };
                InstructionKind::DirectMethod {
                    callee,
                    arguments,
                    called_class,
                }
            }
            23 => InstructionKind::VirtualMethod {
                receiver: Register(self.u32()?),
                slot: MethodSlot(self.u32()?),
                arguments: self.vector(|decoder| Ok(Register(decoder.u32()?)))?,
            },
            24 => InstructionKind::LateStaticMethod {
                receiver: match self.u32()? {
                    NONE => None,
                    receiver => Some(Register(receiver)),
                },
                slot: MethodSlot(self.u32()?),
                arguments: self.vector(|decoder| Ok(Register(decoder.u32()?)))?,
            },
            25 => InstructionKind::ChainPrevious {
                replacement: Register(self.u32()?),
                previous: Register(self.u32()?),
            },
            26 => InstructionKind::InitializeProperty {
                object: Register(self.u32()?),
                property: PropertyId(self.u32()?),
                value: Register(self.u32()?),
            },
            27 => InstructionKind::NewDynamic {
                target: Register(self.u32()?),
                type_arguments: self.vector(|decoder| decoder.ty(0))?,
                arguments: self.vector(|decoder| {
                    let name = match decoder.u8()? {
                        0 => None,
                        1 => Some(decoder.string()?),
                        tag => {
                            return Err(
                                decoder.error(format!("invalid optional argument-name tag {tag}"))
                            );
                        }
                    };
                    Ok(DynamicArgument {
                        name,
                        value: Register(decoder.u32()?),
                        span: decoder.span()?,
                    })
                })?,
            },
            28 => InstructionKind::CheckedNarrow {
                value: Register(self.u32()?),
                narrowed: self.ty(0)?,
            },
            29 => InstructionKind::Closure {
                function: FunctionId(self.u32()?),
                captures: self.vector(|decoder| Ok(Register(decoder.u32()?)))?,
            },
            30 => InstructionKind::CallValue {
                callee: Register(self.u32()?),
                arguments: self.vector(|decoder| Ok(Register(decoder.u32()?)))?,
            },
            tag => return Err(self.error(format!("unknown instruction tag {tag}"))),
        };
        Ok(Instruction {
            destination,
            kind,
            ty,
            span,
        })
    }

    fn constant(&mut self) -> Result<Constant, DecodeError> {
        Ok(match self.u8()? {
            0 => Constant::Integer(i64::from_ne_bytes(self.u64()?.to_ne_bytes())),
            1 => Constant::Float(f64::from_bits(self.u64()?)),
            2 => match self.u8()? {
                0 => Constant::Bool(false),
                1 => Constant::Bool(true),
                value => return Err(self.error(format!("invalid boolean byte {value}"))),
            },
            3 => Constant::Null,
            4 => Constant::String(self.blob()?),
            tag => return Err(self.error(format!("unknown constant tag {tag}"))),
        })
    }

    fn unary(&mut self) -> Result<UnaryOp, DecodeError> {
        match self.u8()? {
            0 => Ok(UnaryOp::Negate),
            1 => Ok(UnaryOp::Not),
            tag => Err(self.error(format!("unknown unary operator tag {tag}"))),
        }
    }

    fn binary(&mut self) -> Result<BinaryOp, DecodeError> {
        Ok(match self.u8()? {
            0 => BinaryOp::Add,
            1 => BinaryOp::Subtract,
            2 => BinaryOp::Multiply,
            3 => BinaryOp::Divide,
            4 => BinaryOp::Remainder,
            5 => BinaryOp::Concatenate,
            6 => BinaryOp::Equal,
            7 => BinaryOp::StrictEqual,
            8 => BinaryOp::NotEqual,
            9 => BinaryOp::Less,
            10 => BinaryOp::LessEqual,
            11 => BinaryOp::Greater,
            12 => BinaryOp::GreaterEqual,
            13 => BinaryOp::And,
            14 => BinaryOp::Or,
            15 => BinaryOp::Coalesce,
            tag => return Err(self.error(format!("unknown binary operator tag {tag}"))),
        })
    }

    fn callee(&mut self) -> Result<Callee, DecodeError> {
        match self.u8()? {
            0 => Ok(Callee::Function(FunctionId(self.u32()?))),
            1 => Ok(Callee::Builtin(Builtin::Count)),
            2 => Ok(Callee::Builtin(Builtin::VarDump)),
            3 => Ok(Callee::Builtin(Builtin::MemoryStreamOpen)),
            4 => Ok(Callee::Builtin(Builtin::TempStreamOpen)),
            5 => Ok(Callee::Builtin(Builtin::StreamsOpen)),
            6 => Ok(Callee::Builtin(Builtin::FilesOpenRead)),
            7 => Ok(Callee::Builtin(Builtin::StreamTell)),
            8 => Ok(Callee::Builtin(Builtin::StreamRead)),
            9 => Ok(Callee::Builtin(Builtin::StreamReadAll)),
            10 => Ok(Callee::Builtin(Builtin::StreamEof)),
            11 => Ok(Callee::Builtin(Builtin::StreamSeek)),
            12 => Ok(Callee::Builtin(Builtin::StreamWriteAll)),
            13 => Ok(Callee::Builtin(Builtin::StreamClose)),
            14 => Ok(Callee::Builtin(Builtin::StreamIsClosed)),
            15 => Ok(Callee::Builtin(Builtin::ExceptionNew)),
            16 => Ok(Callee::Builtin(Builtin::ExceptionGetMessage)),
            17 => Ok(Callee::Builtin(Builtin::ExceptionGetTarget)),
            18 => Ok(Callee::Builtin(Builtin::ExceptionGetSystemCode)),
            19 => Ok(Callee::Builtin(Builtin::ExceptionGetSuppressed)),
            20 => Ok(Callee::Builtin(Builtin::ExceptionConstruct)),
            21 => Ok(Callee::Builtin(Builtin::ExceptionGetCode)),
            22 => Ok(Callee::Builtin(Builtin::ExceptionGetPrevious)),
            23 => ReflectionBuiltin::ALL
                .get(usize::from(self.u8()?))
                .copied()
                .map(|operation| Callee::Builtin(Builtin::Reflection(operation)))
                .ok_or_else(|| self.error("unknown reflection builtin")),
            24 => Ok(Callee::Builtin(Builtin::IsString)),
            25 => Ok(Callee::Builtin(Builtin::IsInt)),
            26 => Ok(Callee::Builtin(Builtin::IsFloat)),
            27 => Ok(Callee::Builtin(Builtin::IsNull)),
            28 => Ok(Callee::Builtin(Builtin::IsNumeric)),
            29 => Ok(Callee::Builtin(Builtin::IsVector)),
            30 => Ok(Callee::Builtin(Builtin::IsMap)),
            31 => Ok(Callee::Builtin(Builtin::IteratorFromCollection)),
            32 => Ok(Callee::Builtin(Builtin::IteratorConstruct)),
            33 => Ok(Callee::Builtin(Builtin::IteratorRewind)),
            34 => Ok(Callee::Builtin(Builtin::IteratorValid)),
            35 => Ok(Callee::Builtin(Builtin::IteratorKey)),
            36 => Ok(Callee::Builtin(Builtin::IteratorValue)),
            37 => Ok(Callee::Builtin(Builtin::IteratorAdvance)),
            38 => Ok(Callee::Builtin(Builtin::IteratorGetInner)),
            39 => Ok(Callee::Builtin(Builtin::IteratorCount)),
            40 => Ok(Callee::Builtin(Builtin::IteratorToVector)),
            41 => Ok(Callee::Builtin(Builtin::IteratorToMap)),
            42 => Ok(Callee::Builtin(Builtin::VectorMap)),
            43 => Ok(Callee::Builtin(Builtin::VectorFilter)),
            44 => Ok(Callee::Builtin(Builtin::VectorSlice)),
            45 => Ok(Callee::Builtin(Builtin::VectorConcat)),
            46 => Ok(Callee::Builtin(Builtin::MapTransform)),
            47 => Ok(Callee::Builtin(Builtin::MapFilter)),
            48 => Ok(Callee::Builtin(Builtin::MapMerge)),
            49 => Ok(Callee::Builtin(Builtin::IteratorApply)),
            50 => Ok(Callee::Builtin(Builtin::CallbackFilterAccept)),
            51 => Ok(Callee::Builtin(Builtin::CallbackFilterConstruct)),
            52 => Ok(Callee::Builtin(Builtin::GeneratorGetReturn)),
            53 => Ok(Callee::Builtin(Builtin::GeneratorClose)),
            54 => Ok(Callee::Builtin(Builtin::OptionSome)),
            55 => Ok(Callee::Builtin(Builtin::OptionNone)),
            56 => Ok(Callee::Builtin(Builtin::OptionIsSome)),
            57 => Ok(Callee::Builtin(Builtin::OptionIsNone)),
            58 => Ok(Callee::Builtin(Builtin::OptionGet)),
            59 => Ok(Callee::Builtin(Builtin::Serialize)),
            60 => Ok(Callee::Builtin(Builtin::Unserialize)),
            61 => Ok(Callee::Builtin(Builtin::TraceLineConstruct)),
            62 => Ok(Callee::Builtin(Builtin::ExceptionGetFile)),
            63 => Ok(Callee::Builtin(Builtin::ExceptionGetLine)),
            64 => Ok(Callee::Builtin(Builtin::ExceptionGetTrace)),
            65 => Ok(Callee::Builtin(Builtin::ExceptionGetTraceAsString)),
            66 => Ok(Callee::Builtin(Builtin::ExceptionToString)),
            67 => Ok(Callee::Builtin(Builtin::LimitSeek)),
            68 => Ok(Callee::Builtin(Builtin::LimitGetPosition)),
            69 => Ok(Callee::Builtin(Builtin::AppendAdd)),
            70 => Ok(Callee::Builtin(Builtin::AppendGetIndex)),
            71 => Ok(Callee::Builtin(Builtin::AppendGetIterator)),
            72 => Ok(Callee::Builtin(Builtin::CachingHasNext)),
            73 => Ok(Callee::Builtin(Builtin::CachingCount)),
            74 => Ok(Callee::Builtin(Builtin::CachingGetCache)),
            75 => Ok(Callee::Builtin(Builtin::CachingGetFlags)),
            76 => Ok(Callee::Builtin(Builtin::CachingSetFlags)),
            77 => Ok(Callee::Builtin(Builtin::CachingOffsetExists)),
            78 => Ok(Callee::Builtin(Builtin::CachingOffsetGet)),
            79 => Ok(Callee::Builtin(Builtin::CachingOffsetSet)),
            80 => Ok(Callee::Builtin(Builtin::CachingOffsetUnset)),
            81 => Ok(Callee::Builtin(Builtin::CachingToString)),
            82 => Ok(Callee::Builtin(Builtin::RecursiveEntryConstruct)),
            83 => Ok(Callee::Builtin(Builtin::RecursiveEntryValue)),
            84 => Ok(Callee::Builtin(Builtin::RecursiveEntryChildren)),
            85 => Ok(Callee::Builtin(Builtin::RecursiveWalkConstruct)),
            86 => Ok(Callee::Builtin(Builtin::RecursiveWalkSetMaxDepth)),
            87 => Ok(Callee::Builtin(Builtin::RecursiveWalkGetMaxDepth)),
            88 => Ok(Callee::Builtin(Builtin::RecursiveWalkGetDepth)),
            89 => Ok(Callee::Builtin(Builtin::RecursiveWalkGetSubIterator)),
            90 => Ok(Callee::Builtin(Builtin::RecursiveWalkGetRecursiveIterator)),
            91 => Ok(Callee::Builtin(Builtin::RecursiveWalkHook)),
            92 => Ok(Callee::Builtin(Builtin::FixedConstruct)),
            93 => Ok(Callee::Builtin(Builtin::FixedSetSize)),
            94 => Ok(Callee::Builtin(Builtin::FixedGetSize)),
            95 => Ok(Callee::Builtin(Builtin::FixedOffsetExists)),
            96 => Ok(Callee::Builtin(Builtin::FixedOffsetGet)),
            97 => Ok(Callee::Builtin(Builtin::FixedOffsetSet)),
            98 => Ok(Callee::Builtin(Builtin::FixedOffsetUnset)),
            99 => Ok(Callee::Builtin(Builtin::FixedGetIterator)),
            100 => Ok(Callee::Builtin(Builtin::ListConstruct)),
            101 => Ok(Callee::Builtin(Builtin::ListPush)),
            102 => Ok(Callee::Builtin(Builtin::ListPop)),
            103 => Ok(Callee::Builtin(Builtin::ListUnshift)),
            104 => Ok(Callee::Builtin(Builtin::ListShift)),
            105 => Ok(Callee::Builtin(Builtin::ListTop)),
            106 => Ok(Callee::Builtin(Builtin::ListBottom)),
            107 => Ok(Callee::Builtin(Builtin::ListCount)),
            108 => Ok(Callee::Builtin(Builtin::ListIsEmpty)),
            109 => Ok(Callee::Builtin(Builtin::ListGetIterator)),
            110 => Ok(Callee::Builtin(Builtin::ListToVector)),
            111 => Ok(Callee::Builtin(Builtin::HeapConstruct)),
            112 => Ok(Callee::Builtin(Builtin::HeapInsert)),
            113 => Ok(Callee::Builtin(Builtin::HeapTop)),
            114 => Ok(Callee::Builtin(Builtin::HeapExtract)),
            115 => Ok(Callee::Builtin(Builtin::HeapCount)),
            116 => Ok(Callee::Builtin(Builtin::HeapIsEmpty)),
            117 => Ok(Callee::Builtin(Builtin::HeapGetIterator)),
            118 => Ok(Callee::Builtin(Builtin::TypedMapConstruct)),
            119 => Ok(Callee::Builtin(Builtin::TypedMapSet)),
            120 => Ok(Callee::Builtin(Builtin::TypedMapGet)),
            121 => Ok(Callee::Builtin(Builtin::TypedMapContains)),
            122 => Ok(Callee::Builtin(Builtin::TypedMapRemove)),
            123 => Ok(Callee::Builtin(Builtin::TypedMapCount)),
            124 => Ok(Callee::Builtin(Builtin::TypedMapToMap)),
            125 => Ok(Callee::Builtin(Builtin::TypedMapGetIterator)),
            126 => Ok(Callee::Builtin(Builtin::ObjectStorageConstruct)),
            127 => Ok(Callee::Builtin(Builtin::ObjectStorageAttach)),
            128 => Ok(Callee::Builtin(Builtin::ObjectStorageGet)),
            129 => Ok(Callee::Builtin(Builtin::ObjectStorageContains)),
            130 => Ok(Callee::Builtin(Builtin::ObjectStorageDetach)),
            131 => Ok(Callee::Builtin(Builtin::ObjectStorageCount)),
            132 => Ok(Callee::Builtin(Builtin::ObjectStorageGetIterator)),
            133 => Ok(Callee::Builtin(Builtin::ParentAccept)),
            134 => Ok(Callee::Builtin(Builtin::RecursiveCallbackConstruct)),
            135 => Ok(Callee::Builtin(Builtin::RecursiveCallbackAccept)),
            tag => Err(self.error(format!("unknown callee tag {tag}"))),
        }
    }

    fn optional_callee(&mut self) -> Result<Option<Callee>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.callee()?)),
            tag => Err(self.error(format!("unknown optional-callee tag {tag}"))),
        }
    }

    fn visibility(&mut self) -> Result<thp_syntax::Visibility, DecodeError> {
        match self.u8()? {
            0 => Ok(thp_syntax::Visibility::Public),
            1 => Ok(thp_syntax::Visibility::Protected),
            2 => Ok(thp_syntax::Visibility::Private),
            tag => Err(self.error(format!("unknown visibility tag {tag}"))),
        }
    }

    fn boolean(&mut self) -> Result<bool, DecodeError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(self.error(format!("invalid boolean byte {value}"))),
        }
    }

    fn terminator(&mut self) -> Result<Terminator, DecodeError> {
        match self.u8()? {
            0 => Ok(Terminator::Jump(BlockId(self.u32()?))),
            1 => Ok(Terminator::Branch {
                condition: Register(self.u32()?),
                then_block: BlockId(self.u32()?),
                else_block: BlockId(self.u32()?),
            }),
            2 => Ok(Terminator::Return(match self.u32()? {
                NONE => None,
                register => Some(Register(register)),
            })),
            3 => Ok(Terminator::Unreachable),
            4 => Ok(Terminator::Throw(Register(self.u32()?))),
            5 => Ok(Terminator::Yield {
                key: match self.u32()? {
                    NONE => None,
                    register => Some(Register(register)),
                },
                value: Register(self.u32()?),
                resume: BlockId(self.u32()?),
            }),
            tag => Err(self.error(format!("unknown terminator tag {tag}"))),
        }
    }

    fn vector<T>(
        &mut self,
        mut decode: impl FnMut(&mut Self) -> Result<T, DecodeError>,
    ) -> Result<Vec<T>, DecodeError> {
        let count = self.len()?;
        let minimum_item_size = 1;
        if count > self.bytes.len().saturating_sub(self.offset) / minimum_item_size {
            return Err(self.error("declared collection length exceeds remaining bytecode"));
        }
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(decode(self)?);
        }
        Ok(values)
    }

    fn error(&self, message: impl Into<String>) -> DecodeError {
        Self::error_at(self.offset, message)
    }

    fn error_at(offset: usize, message: impl Into<String>) -> DecodeError {
        DecodeError {
            offset,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DecodeError, decode};

    #[test]
    fn rejects_short_and_unknown_input_without_panicking() {
        for bytes in [b"".as_slice(), b"THP".as_slice(), b"XXXXXXXX".as_slice()] {
            assert!(matches!(decode(bytes), Err(DecodeError { .. })));
        }
    }
}
