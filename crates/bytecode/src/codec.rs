use std::fmt;

use thp_diagnostics::Span;
use thp_hir::{
    Builtin, CalledClass, Callee, ClassId, FunctionId, LocalId, MethodSlot, NominalKind,
    PropertyId, Type,
};
use thp_mir::{BlockId, Constant, Register};
use thp_syntax::{BinaryOp, UnaryOp};

use crate::{
    BYTECODE_SCHEMA_VERSION, Block, CatchHandler, Class, ExceptionHandler, Function, Instruction,
    InstructionKind, Method, Program, Property, Terminator, verify,
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
        self.len(class.properties.len());
        for property in &class.properties {
            self.ty(&property.ty);
            self.visibility(property.visibility);
            self.u32(property.declaring_class.0);
        }
        self.len(class.methods.len());
        for method in &class.methods {
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
            self.ty(&method.return_type);
        }
        self.len(class.dispatch.len());
        for callee in &class.dispatch {
            self.optional_callee(*callee);
        }
        self.len(class.interfaces.len());
        for interface in &class.interfaces {
            self.u32(interface.0);
        }
        self.u32(class.parent.map_or(NONE, |parent| parent.0));
    }

    fn function(&mut self, function: &Function) {
        self.u32(function.id.0);
        self.string(&function.name);
        self.len(function.parameters.len());
        for parameter in &function.parameters {
            self.u32(parameter.0);
        }
        self.len(function.local_types.len());
        for ty in &function.local_types {
            self.ty(ty);
        }
        self.ty(&function.return_type);
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
        if end < start {
            return Err(self.error("source span ends before it starts"));
        }
        Ok(Span { start, end })
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
        let properties = self.vector(|decoder| {
            Ok(Property {
                ty: decoder.ty(0)?,
                visibility: decoder.visibility()?,
                declaring_class: ClassId(decoder.u32()?),
            })
        })?;
        let methods = self.vector(|decoder| {
            Ok(Method {
                name: decoder.string()?,
                slot: MethodSlot(decoder.u32()?),
                callee: decoder.optional_callee()?,
                visibility: decoder.visibility()?,
                declaring_class: ClassId(decoder.u32()?),
                static_method: decoder.boolean()?,
                abstract_method: decoder.boolean()?,
                final_method: decoder.boolean()?,
                parameter_types: decoder.vector(|decoder| decoder.ty(0))?,
                return_type: decoder.ty(0)?,
            })
        })?;
        let dispatch = self.vector(Self::optional_callee)?;
        let interfaces = self.vector(|decoder| Ok(ClassId(decoder.u32()?)))?;
        let parent = match self.u32()? {
            NONE => None,
            parent => Some(ClassId(parent)),
        };
        Ok(Class {
            id,
            name,
            kind,
            abstract_class,
            final_class,
            properties,
            methods,
            dispatch,
            interfaces,
            parent,
        })
    }

    fn function(&mut self) -> Result<Function, DecodeError> {
        let id = FunctionId(self.u32()?);
        let name = self.string()?;
        let parameters = self.vector(|decoder| Ok(LocalId(decoder.u32()?)))?;
        let local_types = self.vector(|decoder| decoder.ty(0))?;
        let return_type = self.ty(0)?;
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
            parameters,
            local_types,
            return_type,
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
