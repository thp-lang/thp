//! THP's versioned, typed, register-based interpreter contract.

#![allow(clippy::too_many_lines)]

mod codec;

use std::fmt;

use thp_diagnostics::Span;
use thp_hir::{
    Builtin, CalledClass, Callee, ClassId, FunctionId, LocalId, MethodSlot, NominalKind,
    PropertyId, Type,
};
use thp_mir::{BlockId, Constant, Register};
use thp_syntax::{BinaryOp, UnaryOp};

pub use codec::{DecodeError, decode, encode};

pub const BYTECODE_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug)]
pub struct Program {
    pub schema_version: u16,
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
    pub entry: FunctionId,
}

#[derive(Clone, Debug)]
pub struct Class {
    pub id: ClassId,
    pub name: String,
    pub kind: NominalKind,
    pub abstract_class: bool,
    pub final_class: bool,
    pub properties: Vec<Property>,
    pub methods: Vec<Method>,
    pub dispatch: Vec<Option<Callee>>,
    pub interfaces: Vec<ClassId>,
    pub parent: Option<ClassId>,
}

#[derive(Clone, Debug)]
pub struct Property {
    pub ty: Type,
    pub visibility: thp_syntax::Visibility,
    pub declaring_class: ClassId,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub name: String,
    pub slot: MethodSlot,
    pub callee: Option<Callee>,
    pub visibility: thp_syntax::Visibility,
    pub declaring_class: ClassId,
    pub static_method: bool,
    pub abstract_method: bool,
    pub final_method: bool,
    pub parameter_types: Vec<Type>,
    pub return_type: Type,
}

impl Program {
    pub fn instruction_count(&self) -> usize {
        self.functions
            .iter()
            .flat_map(|function| &function.blocks)
            .map(|block| block.instructions.len() + 1)
            .sum()
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: FunctionId,
    pub name: String,
    pub parameters: Vec<LocalId>,
    pub local_types: Vec<Type>,
    pub return_type: Type,
    pub owner: Option<ClassId>,
    pub static_method: bool,
    pub register_types: Vec<Type>,
    pub blocks: Vec<Block>,
    pub exception_handlers: Vec<ExceptionHandler>,
    pub entry: BlockId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ExceptionHandler {
    pub protected_blocks: Vec<BlockId>,
    pub catches: Vec<CatchHandler>,
}

#[derive(Clone, Debug)]
pub struct CatchHandler {
    pub class: Option<ClassId>,
    pub local: LocalId,
    pub target: BlockId,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub destination: Option<Register>,
    pub kind: InstructionKind,
    pub ty: Option<Type>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum InstructionKind {
    Constant(Constant),
    LoadLocal(LocalId),
    StoreLocal {
        local: LocalId,
        value: Register,
    },
    Unary {
        op: UnaryOp,
        operand: Register,
    },
    Binary {
        op: BinaryOp,
        left: Register,
        right: Register,
    },
    IsNull(Register),
    Vector(Vec<Register>),
    Map(Vec<(Register, Register)>),
    Index {
        collection: Register,
        index: Register,
    },
    CollectionLen(Register),
    CollectionKeyAt {
        collection: Register,
        offset: Register,
    },
    CollectionValueAt {
        collection: Register,
        offset: Register,
    },
    SetIndex {
        collection: Register,
        index: Register,
        value: Register,
    },
    Call {
        callee: Callee,
        arguments: Vec<Register>,
    },
    DirectMethod {
        callee: Callee,
        arguments: Vec<Register>,
        called_class: CalledClass,
    },
    VirtualMethod {
        receiver: Register,
        slot: MethodSlot,
        arguments: Vec<Register>,
    },
    LateStaticMethod {
        receiver: Option<Register>,
        slot: MethodSlot,
        arguments: Vec<Register>,
    },
    NewObject(ClassId),
    GetProperty {
        object: Register,
        property: PropertyId,
    },
    SetProperty {
        object: Register,
        property: PropertyId,
        value: Register,
    },
    InitializeProperty {
        object: Register,
        property: PropertyId,
        value: Register,
    },
    InstanceOf {
        value: Register,
        class: ClassId,
    },
    AddSuppressed {
        primary: Register,
        suppressed: Register,
    },
    ChainPrevious {
        replacement: Register,
        previous: Register,
    },
    RaiseUnhandledMatch(Register),
    Phi(Vec<(BlockId, Register)>),
    Print(Register),
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Jump(BlockId),
    Branch {
        condition: Register,
        then_block: BlockId,
        else_block: BlockId,
    },
    Return(Option<Register>),
    Throw(Register),
    Unreachable,
}

pub fn lower(module: &thp_mir::Module) -> Program {
    Program {
        schema_version: BYTECODE_SCHEMA_VERSION,
        entry: module.entry,
        classes: module
            .classes
            .iter()
            .map(|class| Class {
                id: class.id,
                name: class.name.clone(),
                kind: class.kind,
                abstract_class: class.abstract_class,
                final_class: class.final_class,
                properties: class
                    .properties
                    .iter()
                    .map(|property| Property {
                        ty: property.ty.clone(),
                        visibility: property.visibility,
                        declaring_class: property.declaring_class,
                    })
                    .collect(),
                methods: class
                    .methods
                    .iter()
                    .map(|method| Method {
                        name: method.name.clone(),
                        slot: method.slot,
                        callee: method.callee,
                        visibility: method.visibility,
                        declaring_class: method.declaring_class,
                        static_method: method.static_method,
                        abstract_method: method.abstract_method,
                        final_method: method.final_method,
                        parameter_types: method.parameter_types.clone(),
                        return_type: method.return_type.clone(),
                    })
                    .collect(),
                dispatch: class.dispatch.clone(),
                interfaces: class
                    .interfaces
                    .iter()
                    .filter_map(|interface| {
                        module
                            .classes
                            .iter()
                            .find(|candidate| &candidate.name == interface)
                            .map(|interface| interface.id)
                    })
                    .collect(),
                parent: class.parent.as_ref().and_then(|parent| {
                    module
                        .classes
                        .iter()
                        .find(|candidate| &candidate.name == parent)
                        .map(|parent| parent.id)
                }),
            })
            .collect(),
        functions: module
            .functions
            .iter()
            .map(|function| Function {
                id: function.id,
                name: function.name.clone(),
                parameters: function.parameters.clone(),
                local_types: function.local_types.clone(),
                return_type: function.return_type.clone(),
                owner: function.owner,
                static_method: function.static_method,
                register_types: function.register_types.clone(),
                exception_handlers: function
                    .exception_handlers
                    .iter()
                    .map(|handler| ExceptionHandler {
                        protected_blocks: handler.protected_blocks.clone(),
                        catches: handler
                            .catches
                            .iter()
                            .map(|clause| CatchHandler {
                                class: clause.class,
                                local: clause.local,
                                target: clause.target,
                            })
                            .collect(),
                    })
                    .collect(),
                entry: function.entry,
                span: function.span,
                blocks: function
                    .blocks
                    .iter()
                    .map(|block| Block {
                        id: block.id,
                        instructions: block
                            .instructions
                            .iter()
                            .map(|instruction| Instruction {
                                destination: instruction.destination,
                                kind: lower_instruction(&instruction.kind),
                                ty: instruction.ty.clone(),
                                span: instruction.span,
                            })
                            .collect(),
                        terminator: lower_terminator(&block.terminator),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn lower_instruction(instruction: &thp_mir::InstructionKind) -> InstructionKind {
    match instruction {
        thp_mir::InstructionKind::Constant(value) => InstructionKind::Constant(value.clone()),
        thp_mir::InstructionKind::LoadLocal(local) => InstructionKind::LoadLocal(*local),
        thp_mir::InstructionKind::StoreLocal { local, value } => InstructionKind::StoreLocal {
            local: *local,
            value: *value,
        },
        thp_mir::InstructionKind::Unary { op, operand } => InstructionKind::Unary {
            op: *op,
            operand: *operand,
        },
        thp_mir::InstructionKind::Binary { op, left, right } => InstructionKind::Binary {
            op: *op,
            left: *left,
            right: *right,
        },
        thp_mir::InstructionKind::IsNull(register) => InstructionKind::IsNull(*register),
        thp_mir::InstructionKind::Vector(registers) => InstructionKind::Vector(registers.clone()),
        thp_mir::InstructionKind::Map(entries) => InstructionKind::Map(entries.clone()),
        thp_mir::InstructionKind::Index { collection, index } => InstructionKind::Index {
            collection: *collection,
            index: *index,
        },
        thp_mir::InstructionKind::CollectionLen(collection) => {
            InstructionKind::CollectionLen(*collection)
        }
        thp_mir::InstructionKind::CollectionKeyAt { collection, offset } => {
            InstructionKind::CollectionKeyAt {
                collection: *collection,
                offset: *offset,
            }
        }
        thp_mir::InstructionKind::CollectionValueAt { collection, offset } => {
            InstructionKind::CollectionValueAt {
                collection: *collection,
                offset: *offset,
            }
        }
        thp_mir::InstructionKind::SetIndex {
            collection,
            index,
            value,
        } => InstructionKind::SetIndex {
            collection: *collection,
            index: *index,
            value: *value,
        },
        thp_mir::InstructionKind::Call { callee, arguments } => InstructionKind::Call {
            callee: *callee,
            arguments: arguments.clone(),
        },
        thp_mir::InstructionKind::DirectMethod {
            callee,
            arguments,
            called_class,
        } => InstructionKind::DirectMethod {
            callee: *callee,
            arguments: arguments.clone(),
            called_class: *called_class,
        },
        thp_mir::InstructionKind::VirtualMethod {
            receiver,
            slot,
            arguments,
        } => InstructionKind::VirtualMethod {
            receiver: *receiver,
            slot: *slot,
            arguments: arguments.clone(),
        },
        thp_mir::InstructionKind::LateStaticMethod {
            receiver,
            slot,
            arguments,
        } => InstructionKind::LateStaticMethod {
            receiver: *receiver,
            slot: *slot,
            arguments: arguments.clone(),
        },
        thp_mir::InstructionKind::NewObject(class) => InstructionKind::NewObject(*class),
        thp_mir::InstructionKind::GetProperty { object, property } => {
            InstructionKind::GetProperty {
                object: *object,
                property: *property,
            }
        }
        thp_mir::InstructionKind::SetProperty {
            object,
            property,
            value,
        } => InstructionKind::SetProperty {
            object: *object,
            property: *property,
            value: *value,
        },
        thp_mir::InstructionKind::InitializeProperty {
            object,
            property,
            value,
        } => InstructionKind::InitializeProperty {
            object: *object,
            property: *property,
            value: *value,
        },
        thp_mir::InstructionKind::InstanceOf { value, class } => InstructionKind::InstanceOf {
            value: *value,
            class: *class,
        },
        thp_mir::InstructionKind::AddSuppressed {
            primary,
            suppressed,
        } => InstructionKind::AddSuppressed {
            primary: *primary,
            suppressed: *suppressed,
        },
        thp_mir::InstructionKind::ChainPrevious {
            replacement,
            previous,
        } => InstructionKind::ChainPrevious {
            replacement: *replacement,
            previous: *previous,
        },
        thp_mir::InstructionKind::RaiseUnhandledMatch(value) => {
            InstructionKind::RaiseUnhandledMatch(*value)
        }
        thp_mir::InstructionKind::Phi(inputs) => InstructionKind::Phi(inputs.clone()),
        thp_mir::InstructionKind::Print(register) => InstructionKind::Print(*register),
    }
}

fn lower_terminator(terminator: &thp_mir::Terminator) -> Terminator {
    match terminator {
        thp_mir::Terminator::Jump(target) => Terminator::Jump(*target),
        thp_mir::Terminator::Branch {
            condition,
            then_block,
            else_block,
        } => Terminator::Branch {
            condition: *condition,
            then_block: *then_block,
            else_block: *else_block,
        },
        thp_mir::Terminator::Return(value) => Terminator::Return(*value),
        thp_mir::Terminator::Throw(value) => Terminator::Throw(*value),
        thp_mir::Terminator::Unreachable => Terminator::Unreachable,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationError {
    pub function: Option<FunctionId>,
    pub block: Option<BlockId>,
    pub instruction: Option<usize>,
    pub message: String,
}

impl fmt::Display for VerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bytecode verification failed")?;
        if let Some(function) = self.function {
            write!(formatter, " in function #{}", function.0)?;
        }
        if let Some(block) = self.block {
            write!(formatter, ", block {}", block.0)?;
        }
        if let Some(instruction) = self.instruction {
            write!(formatter, ", instruction {instruction}")?;
        }
        write!(formatter, ": {}", self.message)
    }
}

impl std::error::Error for VerificationError {}

/// Verifies structural and static type safety before execution.
///
/// # Errors
///
/// Returns the first invalid function, block, register, type, or control-flow
/// reference.
pub fn verify(program: &Program) -> Result<(), VerificationError> {
    if program.schema_version != BYTECODE_SCHEMA_VERSION {
        return Err(global_error(format!(
            "unsupported schema {}, expected {}",
            program.schema_version, BYTECODE_SCHEMA_VERSION
        )));
    }
    if program.entry.0 as usize >= program.functions.len() {
        return Err(global_error("entry function is out of bounds"));
    }
    for (index, class) in program.classes.iter().enumerate() {
        if program.classes[..index]
            .iter()
            .any(|previous| previous.name == class.name)
        {
            return Err(global_error(format!(
                "nominal type {} is declared more than once",
                class.name
            )));
        }
    }
    for (index, class) in program.classes.iter().enumerate() {
        if class.id.0 as usize != index {
            return Err(global_error(format!(
                "class id {} does not match index {index}",
                class.id.0
            )));
        }
        if class
            .interfaces
            .iter()
            .any(|interface| interface.0 as usize >= program.classes.len())
        {
            return Err(global_error(format!(
                "class {} has an out-of-bounds interface",
                class.name
            )));
        }
        if class
            .parent
            .is_some_and(|parent| parent.0 as usize >= program.classes.len())
        {
            return Err(global_error(format!(
                "class {} has an out-of-bounds parent",
                class.name
            )));
        }
        if let Some(parent) = class.parent {
            if program.classes[parent.0 as usize].kind != class.kind {
                return Err(global_error(format!(
                    "{} {} has a parent of the wrong nominal kind",
                    match class.kind {
                        NominalKind::Class => "class",
                        NominalKind::Interface => "interface",
                        NominalKind::Trait => "trait",
                    },
                    class.name
                )));
            }
            if program.classes[parent.0 as usize].final_class {
                return Err(global_error(format!(
                    "{} extends a final nominal type",
                    class.name
                )));
            }
        }
        if has_parent_cycle(program, class.id) {
            return Err(global_error(format!(
                "{} participates in a nominal inheritance cycle",
                class.name
            )));
        }
        if class.abstract_class && class.final_class {
            return Err(global_error(format!(
                "{} cannot be both abstract and final",
                class.name
            )));
        }
        for interface in &class.interfaces {
            if program.classes[interface.0 as usize].kind != NominalKind::Interface {
                return Err(global_error(format!(
                    "{} has a non-interface in its interface closure",
                    class.name
                )));
            }
        }
        let has_throwable = class
            .interfaces
            .iter()
            .any(|interface| program.classes[interface.0 as usize].name == "Throwable");
        if has_throwable {
            let allowed = match class.kind {
                NominalKind::Interface => class.name == "Throwable",
                NominalKind::Class => {
                    is_instance_of_name(program, &class.name, "Exception")
                        || is_instance_of_name(program, &class.name, "Error")
                }
                NominalKind::Trait => false,
            };
            if !allowed {
                return Err(global_error(format!(
                    "{} violates the sealed Throwable hierarchy",
                    class.name
                )));
            }
        }
        for property in &class.properties {
            if property.declaring_class.0 as usize >= program.classes.len() {
                return Err(global_error(format!(
                    "{} has a property with an invalid declaring class",
                    class.name
                )));
            }
        }
        for method in &class.methods {
            if method.slot.0 as usize >= class.dispatch.len()
                || method.declaring_class.0 as usize >= program.classes.len()
            {
                return Err(global_error(format!(
                    "{} has invalid method metadata",
                    class.name
                )));
            }
            if class.dispatch[method.slot.0 as usize] != method.callee {
                return Err(global_error(format!(
                    "{} has inconsistent dispatch metadata for {}",
                    class.name, method.name
                )));
            }
            if let Some(Callee::Function(function)) = method.callee
                && function.0 as usize >= program.functions.len()
            {
                return Err(global_error(format!(
                    "{} dispatches to an out-of-bounds function",
                    class.name
                )));
            }
        }
        if class.kind == NominalKind::Class
            && !class.abstract_class
            && class
                .methods
                .iter()
                .any(|method| method.abstract_method || method.callee.is_none())
        {
            return Err(global_error(format!(
                "concrete class {} has an incomplete dispatch table",
                class.name
            )));
        }
    }
    verify_descriptor_consistency(program)?;
    for (index, function) in program.functions.iter().enumerate() {
        if function.id.0 as usize != index {
            return Err(function_error(
                function.id,
                format!("function id {} does not match index {index}", function.id.0),
            ));
        }
        if function
            .owner
            .is_some_and(|owner| owner.0 as usize >= program.classes.len())
        {
            return Err(function_error(
                function.id,
                "function owner is out of bounds",
            ));
        }
        verify_function(program, function)?;
    }
    Ok(())
}

fn has_parent_cycle(program: &Program, start: ClassId) -> bool {
    let mut seen = vec![false; program.classes.len()];
    let mut current = Some(start);
    while let Some(class) = current {
        let index = class.0 as usize;
        if seen[index] {
            return true;
        }
        seen[index] = true;
        current = program.classes[index].parent;
    }
    false
}

#[allow(clippy::too_many_lines)]
fn verify_descriptor_consistency(program: &Program) -> Result<(), VerificationError> {
    let dispatch_len = program
        .classes
        .first()
        .map_or(0, |class| class.dispatch.len());
    let mut slot_names = vec![None::<&str>; dispatch_len];
    for class in &program.classes {
        if class.dispatch.len() != dispatch_len {
            return Err(global_error(format!(
                "{} has a dispatch table of inconsistent length",
                class.name
            )));
        }
        if class.kind == NominalKind::Interface {
            if !class.abstract_class {
                return Err(global_error(format!(
                    "interface {} must be abstract",
                    class.name
                )));
            }
            if !class.properties.is_empty() {
                return Err(global_error(format!(
                    "interface {} cannot contain properties",
                    class.name
                )));
            }
        }
        if class
            .interfaces
            .iter()
            .enumerate()
            .any(|(index, interface)| {
                *interface == class.id || class.interfaces[..index].contains(interface)
            })
        {
            return Err(global_error(format!(
                "{} has duplicate or recursive interface metadata",
                class.name
            )));
        }
        for (property_index, property) in class.properties.iter().enumerate() {
            if !is_instance_of_id(program, class.id, property.declaring_class) {
                return Err(global_error(format!(
                    "{} has a property from an unrelated declaring class",
                    class.name
                )));
            }
            if let Some(parent) = class.parent.filter(|_| class.kind == NominalKind::Class) {
                let parent = &program.classes[parent.0 as usize];
                if let Some(inherited) = parent.properties.get(property_index)
                    && (inherited.ty != property.ty
                        || inherited.visibility != property.visibility
                        || inherited.declaring_class != property.declaring_class)
                {
                    return Err(global_error(format!(
                        "{} changes an inherited property slot",
                        class.name
                    )));
                }
            }
        }
        if let Some(parent) = class.parent {
            let parent = &program.classes[parent.0 as usize];
            if class.kind == NominalKind::Class && class.properties.len() < parent.properties.len()
            {
                return Err(global_error(format!(
                    "{} omits inherited property slots",
                    class.name
                )));
            }
            if class.kind == NominalKind::Interface && !class.interfaces.contains(&parent.id) {
                return Err(global_error(format!(
                    "{} omits its parent interface from the closure",
                    class.name
                )));
            }
            if parent
                .interfaces
                .iter()
                .any(|interface| !class.interfaces.contains(interface))
            {
                return Err(global_error(format!(
                    "{} has an incomplete interface closure",
                    class.name
                )));
            }
            for inherited in &parent.methods {
                let Some(method) = class
                    .methods
                    .iter()
                    .find(|method| method.name == inherited.name)
                else {
                    return Err(global_error(format!(
                        "{} omits inherited method {}",
                        class.name, inherited.name
                    )));
                };
                if method.slot != inherited.slot
                    || !method_signature_equal(method, inherited)
                    || visibility_rank(method.visibility) < visibility_rank(inherited.visibility)
                {
                    return Err(global_error(format!(
                        "{} has an incompatible inherited method {}",
                        class.name, inherited.name
                    )));
                }
                if (inherited.final_method
                    || inherited.visibility == thp_syntax::Visibility::Private)
                    && (method.callee != inherited.callee
                        || method.declaring_class != inherited.declaring_class)
                {
                    return Err(global_error(format!(
                        "{} replaces final or private method {}",
                        class.name, inherited.name
                    )));
                }
            }
        }
        for interface in &class.interfaces {
            let interface = &program.classes[interface.0 as usize];
            for requirement in &interface.methods {
                let Some(method) = class
                    .methods
                    .iter()
                    .find(|method| method.name == requirement.name)
                else {
                    return Err(global_error(format!(
                        "{} omits interface method {}",
                        class.name, requirement.name
                    )));
                };
                if !method_signature_equal(method, requirement)
                    || method.visibility != thp_syntax::Visibility::Public
                {
                    return Err(global_error(format!(
                        "{} does not satisfy interface method {}",
                        class.name, requirement.name
                    )));
                }
            }
        }
        for (index, method) in class.methods.iter().enumerate() {
            if class.methods[..index]
                .iter()
                .any(|previous| previous.name == method.name || previous.slot == method.slot)
            {
                return Err(global_error(format!(
                    "{} has duplicate method metadata",
                    class.name
                )));
            }
            let slot = method.slot.0 as usize;
            if let Some(previous) = slot_names[slot] {
                if previous != method.name {
                    return Err(global_error(format!(
                        "method slot {} names both {} and {}",
                        method.slot.0, previous, method.name
                    )));
                }
            } else {
                slot_names[slot] = Some(&method.name);
            }
            if method.abstract_method && method.callee.is_some() {
                return Err(global_error(format!(
                    "{} has an implemented abstract method {}",
                    class.name, method.name
                )));
            }
            if method.abstract_method
                && (method.final_method || method.visibility == thp_syntax::Visibility::Private)
            {
                return Err(global_error(format!(
                    "{} has an invalid abstract method {}",
                    class.name, method.name
                )));
            }
            if class.kind == NominalKind::Interface
                && (!method.abstract_method
                    || method.callee.is_some()
                    || method.visibility != thp_syntax::Visibility::Public)
            {
                return Err(global_error(format!(
                    "interface {} has invalid method {}",
                    class.name, method.name
                )));
            }
            if class.kind == NominalKind::Class
                && !method.abstract_method
                && method.callee.is_none()
            {
                return Err(global_error(format!(
                    "{} has a concrete method without an implementation",
                    class.name
                )));
            }
            if let Some(Callee::Function(function)) = method.callee {
                verify_user_method(program, method, function)?;
            }
        }
        for (slot, callee) in class.dispatch.iter().enumerate() {
            if callee.is_some()
                && !class
                    .methods
                    .iter()
                    .any(|method| method.slot.0 as usize == slot && method.callee == *callee)
            {
                return Err(global_error(format!(
                    "{} dispatches through an undescribed slot",
                    class.name
                )));
            }
        }
    }
    Ok(())
}

fn method_signature_equal(left: &Method, right: &Method) -> bool {
    left.static_method == right.static_method
        && left.parameter_types == right.parameter_types
        && left.return_type == right.return_type
}

const fn visibility_rank(visibility: thp_syntax::Visibility) -> u8 {
    match visibility {
        thp_syntax::Visibility::Private => 0,
        thp_syntax::Visibility::Protected => 1,
        thp_syntax::Visibility::Public => 2,
    }
}

fn verify_user_method(
    program: &Program,
    method: &Method,
    function: FunctionId,
) -> Result<(), VerificationError> {
    let function = &program.functions[function.0 as usize];
    if function.owner != Some(method.declaring_class)
        || function.static_method != method.static_method
        || function.return_type != method.return_type
    {
        return Err(global_error(format!(
            "method {} disagrees with its function metadata",
            method.name
        )));
    }
    let expected_count = method.parameter_types.len() + usize::from(!method.static_method);
    if function.parameters.len() != expected_count {
        return Err(global_error(format!(
            "method {} has an incompatible function parameter count",
            method.name
        )));
    }
    if function
        .parameters
        .iter()
        .any(|local| local.0 as usize >= function.local_types.len())
    {
        return Err(global_error(format!(
            "method {} has an out-of-bounds function parameter",
            method.name
        )));
    }
    let mut parameters = function.parameters.iter();
    if !method.static_method {
        let receiver = parameters.next().expect("receiver count was checked");
        let declaring = &program.classes[method.declaring_class.0 as usize];
        if function.local_types[receiver.0 as usize] != Type::Object(declaring.name.clone()) {
            return Err(global_error(format!(
                "method {} has an incompatible receiver",
                method.name
            )));
        }
    }
    if parameters
        .zip(&method.parameter_types)
        .any(|(local, expected)| function.local_types[local.0 as usize] != *expected)
    {
        return Err(global_error(format!(
            "method {} disagrees with its declared signature",
            method.name
        )));
    }
    Ok(())
}

fn verify_function(program: &Program, function: &Function) -> Result<(), VerificationError> {
    if function.blocks.is_empty() {
        return Err(function_error(function.id, "function has no blocks"));
    }
    if function.entry.0 as usize >= function.blocks.len() {
        return Err(function_error(function.id, "entry block is out of bounds"));
    }
    for parameter in &function.parameters {
        if parameter.0 as usize >= function.local_types.len() {
            return Err(function_error(
                function.id,
                format!("parameter local {} is out of bounds", parameter.0),
            ));
        }
    }
    for (index, block) in function.blocks.iter().enumerate() {
        if block.id.0 as usize != index {
            return Err(block_error(
                function.id,
                block.id,
                format!("block id {} does not match index {index}", block.id.0),
            ));
        }
        for (instruction_index, instruction) in block.instructions.iter().enumerate() {
            verify_instruction(program, function, block.id, instruction_index, instruction)?;
        }
        verify_terminator(program, function, block)?;
    }
    for handler in &function.exception_handlers {
        if handler.protected_blocks.is_empty() {
            return Err(function_error(
                function.id,
                "exception handler has no protected blocks",
            ));
        }
        if handler.catches.is_empty() {
            return Err(function_error(
                function.id,
                "exception handler has no catch clauses",
            ));
        }
        for protected in &handler.protected_blocks {
            check_block(function, *protected, function.entry, None)?;
        }
        for (clause_index, clause) in handler.catches.iter().enumerate() {
            check_block(function, clause.target, function.entry, None)?;
            check_local(function, clause.local, function.entry, None)?;
            if handler.protected_blocks.contains(&clause.target) {
                return Err(function_error(
                    function.id,
                    "an exception handler cannot protect its own catch target",
                ));
            }
            if let Some(class) = clause.class {
                let Some(class) = program.classes.get(class.0 as usize) else {
                    return Err(function_error(function.id, "catch class is out of bounds"));
                };
                if !is_instance_of_name(program, &class.name, "Throwable") {
                    return Err(function_error(
                        function.id,
                        "catch class is not a Throwable subtype",
                    ));
                }
                if function.local_types[clause.local.0 as usize] != Type::Object(class.name.clone())
                {
                    return Err(function_error(
                        function.id,
                        "catch local type does not match catch class",
                    ));
                }
                for previous in &handler.catches[..clause_index] {
                    match previous.class {
                        None => {
                            return Err(function_error(
                                function.id,
                                "a catch clause follows an earlier catch-all",
                            ));
                        }
                        Some(previous) if is_instance_of_id(program, class.id, previous) => {
                            return Err(function_error(
                                function.id,
                                "a catch clause is already subsumed by an earlier clause",
                            ));
                        }
                        Some(_) => {}
                    }
                }
            } else if function.local_types[clause.local.0 as usize]
                != Type::Object("Throwable".to_owned())
            {
                return Err(function_error(
                    function.id,
                    "catch-all local must have type Throwable",
                ));
            } else if handler.catches[..clause_index]
                .iter()
                .any(|previous| previous.class.is_none())
            {
                return Err(function_error(
                    function.id,
                    "a catch-all follows an earlier catch-all",
                ));
            }
        }
    }
    for (index, earlier) in function.exception_handlers.iter().enumerate() {
        for later in &function.exception_handlers[index + 1..] {
            let overlaps = earlier
                .protected_blocks
                .iter()
                .any(|block| later.protected_blocks.contains(block));
            if !overlaps {
                continue;
            }
            let earlier_contains_later = later
                .protected_blocks
                .iter()
                .all(|block| earlier.protected_blocks.contains(block));
            let later_contains_earlier = earlier
                .protected_blocks
                .iter()
                .all(|block| later.protected_blocks.contains(block));
            if earlier_contains_later {
                return Err(function_error(
                    function.id,
                    "exception regions must be ordered innermost first",
                ));
            }
            if !later_contains_earlier {
                return Err(function_error(
                    function.id,
                    "exception regions must be disjoint or properly nested",
                ));
            }
        }
    }
    Ok(())
}

fn verify_instruction(
    program: &Program,
    function: &Function,
    block: BlockId,
    index: usize,
    instruction: &Instruction,
) -> Result<(), VerificationError> {
    match (instruction.destination, &instruction.ty) {
        (Some(destination), Some(ty)) => {
            check_register(function, destination, block, Some(index))?;
            if &function.register_types[destination.0 as usize] != ty {
                return Err(instruction_error(
                    function.id,
                    block,
                    index,
                    "destination type does not match the register type table",
                ));
            }
        }
        (None, None) => {}
        _ => {
            return Err(instruction_error(
                function.id,
                block,
                index,
                "destination and result type must either both be present or both be absent",
            ));
        }
    }

    let error = |message: &str| instruction_error(function.id, block, index, message);
    match &instruction.kind {
        InstructionKind::Constant(constant) => {
            let actual = constant_type(constant);
            if instruction.ty.as_ref() != Some(&actual) {
                return Err(error(&format!(
                    "constant type `{actual}` does not match instruction result"
                )));
            }
        }
        InstructionKind::LoadLocal(local) => {
            check_local(function, *local, block, Some(index))?;
            if instruction.ty.as_ref() != Some(&function.local_types[local.0 as usize]) {
                return Err(error("loaded local type does not match result type"));
            }
        }
        InstructionKind::StoreLocal { local, value } => {
            check_local(function, *local, block, Some(index))?;
            check_register(function, *value, block, Some(index))?;
            if !type_accepts(
                program,
                &function.local_types[local.0 as usize],
                &function.register_types[value.0 as usize],
            ) {
                return Err(error("stored value type does not match local type"));
            }
            if instruction.destination.is_some() {
                return Err(error("store instruction cannot have a destination"));
            }
        }
        InstructionKind::Unary { operand, .. } | InstructionKind::IsNull(operand) => {
            check_register(function, *operand, block, Some(index))?;
        }
        InstructionKind::Binary { op, left, right } => {
            check_register(function, *left, block, Some(index))?;
            check_register(function, *right, block, Some(index))?;
            if *op == BinaryOp::Concatenate {
                if !function.register_types[left.0 as usize].is_output_scalar()
                    || !function.register_types[right.0 as usize].is_output_scalar()
                {
                    return Err(error("concatenation operands must be output scalars"));
                }
                if instruction.ty.as_ref() != Some(&Type::String) {
                    return Err(error("concatenation must produce string"));
                }
            }
        }
        InstructionKind::Vector(registers) => {
            let Some(Type::Vector(element)) = instruction.ty.as_ref() else {
                return Err(error("vector instruction must produce a vector type"));
            };
            for register in registers {
                check_register(function, *register, block, Some(index))?;
                if !type_accepts(
                    program,
                    element,
                    &function.register_types[register.0 as usize],
                ) {
                    return Err(error("vector element type does not match result type"));
                }
            }
        }
        InstructionKind::Map(entries) => {
            let Some(Type::Map(key_type, value_type)) = instruction.ty.as_ref() else {
                return Err(error("map instruction must produce a map type"));
            };
            for (key, value) in entries {
                check_register(function, *key, block, Some(index))?;
                check_register(function, *value, block, Some(index))?;
                if !type_accepts(program, key_type, &function.register_types[key.0 as usize])
                    || !type_accepts(
                        program,
                        value_type,
                        &function.register_types[value.0 as usize],
                    )
                {
                    return Err(error("map entry type does not match result type"));
                }
            }
        }
        InstructionKind::Index {
            collection,
            index: key,
        } => {
            check_register(function, *collection, block, Some(index))?;
            check_register(function, *key, block, Some(index))?;
            let (expected_key, expected_result) =
                match &function.register_types[collection.0 as usize] {
                    Type::Vector(value) => (&Type::Int, value.as_ref()),
                    Type::Map(key, value) => (key.as_ref(), value.as_ref()),
                    Type::String => (&Type::Int, &Type::String),
                    _ => return Err(error("index instruction requires an indexable value")),
                };
            if !type_accepts(
                program,
                expected_key,
                &function.register_types[key.0 as usize],
            ) || instruction.ty.as_ref() != Some(expected_result)
            {
                return Err(error("index operand or result type is incorrect"));
            }
        }
        InstructionKind::CollectionLen(collection) => {
            check_register(function, *collection, block, Some(index))?;
            if !matches!(
                function.register_types[collection.0 as usize],
                Type::Vector(_) | Type::Map(_, _)
            ) || instruction.ty.as_ref() != Some(&Type::Int)
            {
                return Err(error("invalid collection-length instruction"));
            }
        }
        InstructionKind::CollectionKeyAt { collection, offset } => {
            check_register(function, *collection, block, Some(index))?;
            check_register(function, *offset, block, Some(index))?;
            if function.register_types[offset.0 as usize] != Type::Int {
                return Err(error("collection iteration offset must be int"));
            }
            let expected = match &function.register_types[collection.0 as usize] {
                Type::Vector(_) => Type::Int,
                Type::Map(key, _) => key.as_ref().clone(),
                _ => return Err(error("collection-key instruction requires a collection")),
            };
            if instruction.ty.as_ref() != Some(&expected) {
                return Err(error("collection-key result type is incorrect"));
            }
        }
        InstructionKind::CollectionValueAt { collection, offset } => {
            check_register(function, *collection, block, Some(index))?;
            check_register(function, *offset, block, Some(index))?;
            if function.register_types[offset.0 as usize] != Type::Int {
                return Err(error("collection iteration offset must be int"));
            }
            let expected = match &function.register_types[collection.0 as usize] {
                Type::Vector(value) | Type::Map(_, value) => value.as_ref().clone(),
                _ => return Err(error("collection-value instruction requires a collection")),
            };
            if instruction.ty.as_ref() != Some(&expected) {
                return Err(error("collection-value result type is incorrect"));
            }
        }
        InstructionKind::SetIndex {
            collection,
            index: key,
            value,
        } => {
            check_register(function, *collection, block, Some(index))?;
            check_register(function, *key, block, Some(index))?;
            check_register(function, *value, block, Some(index))?;
            let collection_type = &function.register_types[collection.0 as usize];
            let (key_type, value_type) = match collection_type {
                Type::Vector(value) => (Type::Int, value.as_ref().clone()),
                Type::Map(key, value) => (key.as_ref().clone(), value.as_ref().clone()),
                _ => return Err(error("set-index instruction requires a collection")),
            };
            if !type_accepts(program, &key_type, &function.register_types[key.0 as usize])
                || !type_accepts(
                    program,
                    &value_type,
                    &function.register_types[value.0 as usize],
                )
                || instruction.ty.as_ref() != Some(collection_type)
            {
                return Err(error("set-index operand or result type is incorrect"));
            }
        }
        InstructionKind::Call { callee, arguments } => {
            for argument in arguments {
                check_register(function, *argument, block, Some(index))?;
            }
            if let Callee::Function(target) = callee
                && program
                    .functions
                    .get(target.0 as usize)
                    .is_some_and(|target| target.owner.is_some())
            {
                return Err(error("plain call cannot bypass method dispatch"));
            }
            verify_callee_call(
                program,
                *callee,
                arguments,
                instruction.ty.as_ref(),
                function,
            )
            .map_err(|message| error(&message))?;
        }
        InstructionKind::DirectMethod {
            callee,
            arguments,
            called_class,
        } => {
            for argument in arguments {
                check_register(function, *argument, block, Some(index))?;
            }
            match called_class {
                CalledClass::Explicit(class) if class.0 as usize >= program.classes.len() => {
                    return Err(error("direct method called class is out of bounds"));
                }
                CalledClass::Forwarded if function.owner.is_none() => {
                    return Err(error(
                        "forwarding direct method call requires a lexical owner",
                    ));
                }
                CalledClass::Receiver => {
                    let Some(receiver) = arguments.first() else {
                        return Err(error("receiver-derived direct method call has no receiver"));
                    };
                    if !matches!(
                        function.register_types[receiver.0 as usize],
                        Type::Object(_)
                    ) {
                        return Err(error(
                            "receiver-derived called class requires an object receiver",
                        ));
                    }
                }
                CalledClass::Explicit(_) | CalledClass::Forwarded => {}
            }
            let Some(method) = direct_method(program, function, *callee, *called_class, arguments)
            else {
                return Err(error("direct method call violates member visibility"));
            };
            let signature_arguments = if method.static_method {
                arguments.as_slice()
            } else {
                arguments
                    .get(1..)
                    .ok_or_else(|| error("instance method call has no receiver"))?
            };
            verify_method_signature(
                program,
                method,
                signature_arguments,
                instruction.ty.as_ref(),
                function,
            )
            .map_err(|message| error(&message))?;
            verify_callee_call(
                program,
                *callee,
                arguments,
                instruction.ty.as_ref(),
                function,
            )
            .map_err(|message| error(&message))?;
        }
        InstructionKind::VirtualMethod {
            receiver,
            slot,
            arguments,
        } => {
            check_register(function, *receiver, block, Some(index))?;
            for argument in arguments {
                check_register(function, *argument, block, Some(index))?;
            }
            let Type::Object(class_name) = &function.register_types[receiver.0 as usize] else {
                return Err(error("virtual method receiver is not an object"));
            };
            let class = class_by_name(program, class_name)
                .ok_or_else(|| error("virtual receiver class is unavailable"))?;
            let method = class
                .methods
                .iter()
                .find(|method| method.slot == *slot)
                .ok_or_else(|| error("virtual method slot is unavailable"))?;
            if method.static_method {
                return Err(error("virtual instance call targets a static method"));
            }
            verify_method_signature(
                program,
                method,
                arguments,
                instruction.ty.as_ref(),
                function,
            )
            .map_err(|message| error(&message))?;
            if !member_accessible(program, function.owner, method) {
                return Err(error("virtual method call violates member visibility"));
            }
        }
        InstructionKind::LateStaticMethod {
            receiver,
            slot,
            arguments,
        } => {
            if let Some(receiver) = receiver {
                check_register(function, *receiver, block, Some(index))?;
            }
            for argument in arguments {
                check_register(function, *argument, block, Some(index))?;
            }
            let owner = function
                .owner
                .and_then(|owner| program.classes.get(owner.0 as usize))
                .ok_or_else(|| error("late-static call requires a lexical owner"))?;
            let method = owner
                .methods
                .iter()
                .find(|method| method.slot == *slot)
                .ok_or_else(|| error("late-static method slot is unavailable"))?;
            if method.static_method == receiver.is_some() {
                return Err(error(
                    "late-static receiver does not match method staticness",
                ));
            }
            verify_method_signature(
                program,
                method,
                arguments,
                instruction.ty.as_ref(),
                function,
            )
            .map_err(|message| error(&message))?;
            if !member_accessible(program, function.owner, method) {
                return Err(error("late-static call violates member visibility"));
            }
        }
        InstructionKind::NewObject(class) => {
            let Some(class) = program.classes.get(class.0 as usize) else {
                return Err(error("allocated class is out of bounds"));
            };
            if class.kind != NominalKind::Class || class.abstract_class {
                return Err(error("only a concrete class can be allocated"));
            }
            if instruction.ty.as_ref() != Some(&Type::Object(class.name.clone())) {
                return Err(error("allocated object type does not match its class"));
            }
        }
        InstructionKind::GetProperty { object, property } => {
            check_register(function, *object, block, Some(index))?;
            let Type::Object(class_name) = &function.register_types[object.0 as usize] else {
                return Err(error("property receiver is not an object"));
            };
            let Some(class) = program
                .classes
                .iter()
                .find(|class| &class.name == class_name)
            else {
                return Err(error("property receiver class is unavailable"));
            };
            let Some(property) = class.properties.get(property.0 as usize) else {
                return Err(error("loaded property is out of bounds"));
            };
            if !property_accessible(program, function.owner, property) {
                return Err(error("property load violates member visibility"));
            }
            if instruction.ty.as_ref() != Some(&property.ty) {
                return Err(error("loaded property type does not match result type"));
            }
        }
        InstructionKind::SetProperty {
            object,
            property,
            value,
        } => {
            check_register(function, *object, block, Some(index))?;
            check_register(function, *value, block, Some(index))?;
            let Type::Object(class_name) = &function.register_types[object.0 as usize] else {
                return Err(error("property receiver is not an object"));
            };
            let Some(class) = program
                .classes
                .iter()
                .find(|class| &class.name == class_name)
            else {
                return Err(error("property receiver class is unavailable"));
            };
            let Some(property) = class.properties.get(property.0 as usize) else {
                return Err(error("stored property is out of bounds"));
            };
            if !property_accessible(program, function.owner, property) {
                return Err(error("property store violates member visibility"));
            }
            if !type_accepts(
                program,
                &property.ty,
                &function.register_types[value.0 as usize],
            ) {
                return Err(error("stored property type does not match property type"));
            }
            if instruction.destination.is_some() {
                return Err(error("property store cannot have a destination"));
            }
        }
        InstructionKind::InitializeProperty {
            object,
            property,
            value,
        } => {
            check_register(function, *object, block, Some(index))?;
            check_register(function, *value, block, Some(index))?;
            let Type::Object(class_name) = &function.register_types[object.0 as usize] else {
                return Err(error("property initializer receiver is not an object"));
            };
            let class = class_by_name(program, class_name)
                .ok_or_else(|| error("property initializer class is unavailable"))?;
            let allocated_here = function.blocks.iter().any(|candidate| {
                candidate.instructions.iter().any(|candidate| {
                    candidate.destination == Some(*object)
                        && matches!(
                            candidate.kind,
                            InstructionKind::NewObject(allocated) if allocated == class.id
                        )
                })
            });
            if !allocated_here {
                return Err(error(
                    "property initialization requires a freshly allocated receiver",
                ));
            }
            let property = class
                .properties
                .get(property.0 as usize)
                .ok_or_else(|| error("initialized property is out of bounds"))?;
            if !type_accepts(
                program,
                &property.ty,
                &function.register_types[value.0 as usize],
            ) {
                return Err(error("initialized value type does not match property type"));
            }
            if instruction.destination.is_some() {
                return Err(error("property initializer cannot have a destination"));
            }
        }
        InstructionKind::InstanceOf { value, class } => {
            check_register(function, *value, block, Some(index))?;
            if program.classes.get(class.0 as usize).is_none() {
                return Err(error("instanceof class is out of bounds"));
            }
            if instruction.ty.as_ref() != Some(&Type::Bool) {
                return Err(error("instanceof result is not bool"));
            }
        }
        InstructionKind::AddSuppressed {
            primary,
            suppressed,
        } => {
            check_register(function, *primary, block, Some(index))?;
            check_register(function, *suppressed, block, Some(index))?;
            let throwable = Type::Object("Throwable".to_owned());
            if !type_accepts(
                program,
                &throwable,
                &function.register_types[primary.0 as usize],
            ) || !type_accepts(
                program,
                &throwable,
                &function.register_types[suppressed.0 as usize],
            ) {
                return Err(error("suppressed failures must be Throwable values"));
            }
            if instruction.destination.is_some() {
                return Err(error("add-suppressed cannot have a destination"));
            }
        }
        InstructionKind::ChainPrevious {
            replacement,
            previous,
        } => {
            check_register(function, *replacement, block, Some(index))?;
            check_register(function, *previous, block, Some(index))?;
            let throwable = Type::Object("Throwable".to_owned());
            if !type_accepts(
                program,
                &throwable,
                &function.register_types[replacement.0 as usize],
            ) || !type_accepts(
                program,
                &throwable,
                &function.register_types[previous.0 as usize],
            ) {
                return Err(error("previous-chain operands must be Throwable values"));
            }
            if instruction.destination.is_some() {
                return Err(error("chain-previous cannot have a destination"));
            }
        }
        InstructionKind::RaiseUnhandledMatch(value) => {
            check_register(function, *value, block, Some(index))?;
        }
        InstructionKind::Phi(inputs) => {
            let Some(result_type) = instruction.ty.as_ref() else {
                return Err(error("phi requires a result"));
            };
            if inputs.is_empty() {
                return Err(error("phi requires at least one predecessor"));
            }
            for (predecessor, register) in inputs {
                check_block(function, *predecessor, block, Some(index))?;
                check_register(function, *register, block, Some(index))?;
                if !type_accepts(
                    program,
                    result_type,
                    &function.register_types[register.0 as usize],
                ) {
                    return Err(error("phi input type does not match result type"));
                }
            }
        }
        InstructionKind::Print(register) => {
            check_register(function, *register, block, Some(index))?;
            if !function.register_types[register.0 as usize].is_output_scalar() {
                return Err(error("print operand must be an output scalar"));
            }
            if instruction.destination.is_some() {
                return Err(error("print instruction cannot have a destination"));
            }
        }
    }
    Ok(())
}

fn verify_callee_call(
    program: &Program,
    callable: Callee,
    arguments: &[Register],
    result: Option<&Type>,
    calling_function: &Function,
) -> Result<(), String> {
    match callable {
        Callee::Function(target) => {
            let target = program
                .functions
                .get(target.0 as usize)
                .ok_or_else(|| format!("called function {} is out of bounds", target.0))?;
            if arguments.len() != target.parameters.len() {
                return Err("call argument count does not match function signature".to_owned());
            }
            for (argument, parameter) in arguments.iter().zip(&target.parameters) {
                if !type_accepts(
                    program,
                    &target.local_types[parameter.0 as usize],
                    &calling_function.register_types[argument.0 as usize],
                ) {
                    return Err("call argument type does not match parameter type".to_owned());
                }
            }
            if result != Some(&target.return_type) {
                return Err("call result type does not match function return type".to_owned());
            }
            Ok(())
        }
        Callee::Builtin(builtin) => {
            verify_builtin_call(program, builtin, arguments, result, calling_function)
        }
    }
}

fn verify_method_signature(
    program: &Program,
    method: &Method,
    arguments: &[Register],
    result: Option<&Type>,
    caller: &Function,
) -> Result<(), String> {
    if arguments.len() != method.parameter_types.len() {
        return Err("method argument count does not match signature".to_owned());
    }
    for (argument, parameter) in arguments.iter().zip(&method.parameter_types) {
        if !type_accepts(
            program,
            parameter,
            &caller.register_types[argument.0 as usize],
        ) {
            return Err("method argument type does not match signature".to_owned());
        }
    }
    if result != Some(&method.return_type) {
        return Err("method result type does not match signature".to_owned());
    }
    Ok(())
}

fn class_by_name<'program>(program: &'program Program, name: &str) -> Option<&'program Class> {
    program.classes.iter().find(|class| class.name == name)
}

fn member_accessible(program: &Program, owner: Option<ClassId>, method: &Method) -> bool {
    visibility_accessible(program, owner, method.declaring_class, method.visibility)
}

fn property_accessible(program: &Program, owner: Option<ClassId>, property: &Property) -> bool {
    visibility_accessible(
        program,
        owner,
        property.declaring_class,
        property.visibility,
    )
}

fn visibility_accessible(
    program: &Program,
    owner: Option<ClassId>,
    declaring: ClassId,
    visibility: thp_syntax::Visibility,
) -> bool {
    match visibility {
        thp_syntax::Visibility::Public => true,
        thp_syntax::Visibility::Private => owner == Some(declaring),
        thp_syntax::Visibility::Protected => owner.is_some_and(|owner| {
            owner == declaring || is_instance_of_id(program, owner, declaring)
        }),
    }
}

fn direct_method<'program>(
    program: &'program Program,
    calling_function: &Function,
    callable: Callee,
    called_class: CalledClass,
    arguments: &[Register],
) -> Option<&'program Method> {
    program
        .classes
        .iter()
        .flat_map(|class| &class.methods)
        .filter(|method| method.callee == Some(callable))
        .find(|method| {
            if !member_accessible(program, calling_function.owner, method) {
                return false;
            }
            let context_matches = match called_class {
                CalledClass::Explicit(class) => {
                    is_instance_of_id(program, class, method.declaring_class)
                }
                CalledClass::Forwarded => calling_function
                    .owner
                    .is_some_and(|owner| is_instance_of_id(program, owner, method.declaring_class)),
                CalledClass::Receiver => arguments.first().is_some_and(|receiver| {
                    let Type::Object(name) = &calling_function.register_types[receiver.0 as usize]
                    else {
                        return false;
                    };
                    class_by_name(program, name).is_some_and(|class| {
                        is_instance_of_id(program, class.id, method.declaring_class)
                    })
                }),
            };
            if !context_matches {
                return false;
            }
            if method.static_method {
                return called_class != CalledClass::Receiver;
            }
            arguments.first().is_some_and(|receiver| {
                let Type::Object(name) = &calling_function.register_types[receiver.0 as usize]
                else {
                    return false;
                };
                class_by_name(program, name).is_some_and(|class| {
                    is_instance_of_id(program, class.id, method.declaring_class)
                })
            })
        })
}

fn verify_builtin_call(
    program: &Program,
    builtin: Builtin,
    arguments: &[Register],
    result: Option<&Type>,
    function: &Function,
) -> Result<(), String> {
    let argument_type = |index: usize| &function.register_types[arguments[index].0 as usize];
    let check = |expected_arguments: &[Type], expected_result: &Type| {
        if arguments.len() != expected_arguments.len() {
            return Err(format!(
                "builtin requires {} arguments, found {}",
                expected_arguments.len(),
                arguments.len()
            ));
        }
        for (index, expected) in expected_arguments.iter().enumerate() {
            if argument_type(index) != expected {
                return Err(format!("builtin argument {index} has the wrong type"));
            }
        }
        if result != Some(expected_result) {
            return Err("builtin result has the wrong type".to_owned());
        }
        Ok(())
    };
    let object_receiver = |expected: &str| {
        arguments
            .first()
            .is_some_and(|register| {
                type_accepts(
                    program,
                    &Type::Object(expected.to_owned()),
                    &function.register_types[register.0 as usize],
                )
            })
            .then_some(())
            .ok_or_else(|| format!("builtin requires a {expected} receiver"))
    };
    match builtin {
        Builtin::Count => {
            if arguments.len() != 1 {
                return Err("`count` requires one argument".to_owned());
            }
            if result != Some(&Type::Int) {
                return Err("`count` result must be int".to_owned());
            }
            Ok(())
        }
        Builtin::VarDump => {
            if result != Some(&Type::Void) {
                return Err("`var_dump` result must be void".to_owned());
            }
            Ok(())
        }
        Builtin::MemoryStreamOpen => {
            check(&[Type::String], &Type::Object("MemoryStream".to_owned()))
        }
        Builtin::TempStreamOpen => check(&[Type::Int], &Type::Object("TempStream".to_owned())),
        Builtin::StreamsOpen => {
            if arguments.len() != 2
                || argument_type(0) != &Type::String
                || argument_type(1) != &Type::Int
                || !matches!(
                    result,
                    Some(Type::Object(name))
                        if matches!(
                            name.as_str(),
                            "MemoryStream" | "TempStream" | "ReadableStream"
                        )
                )
            {
                return Err("invalid `Streams::open` signature".to_owned());
            }
            Ok(())
        }
        Builtin::FilesOpenRead => check(
            &[Type::String],
            &Type::Object("ReadableFileStream".to_owned()),
        ),
        Builtin::StreamTell => {
            object_receiver("ReadableStream")?;
            if arguments.len() != 1 || result != Some(&Type::Int) {
                return Err("invalid stream `tell` signature".to_owned());
            }
            Ok(())
        }
        Builtin::StreamRead => {
            object_receiver("ReadableStream")?;
            if arguments.len() != 2
                || argument_type(1) != &Type::Int
                || result != Some(&Type::String)
            {
                return Err("invalid stream `read` signature".to_owned());
            }
            Ok(())
        }
        Builtin::StreamReadAll => {
            object_receiver("ReadableStream")?;
            if arguments.len() != 2
                || !matches!(argument_type(1), Type::Int | Type::Null)
                || result != Some(&Type::String)
            {
                return Err("invalid stream `readAll` signature".to_owned());
            }
            Ok(())
        }
        Builtin::StreamEof | Builtin::StreamIsClosed => {
            object_receiver(if builtin == Builtin::StreamEof {
                "ReadableStream"
            } else {
                "Closeable"
            })?;
            if arguments.len() != 1 || result != Some(&Type::Bool) {
                return Err("invalid boolean stream method signature".to_owned());
            }
            Ok(())
        }
        Builtin::StreamSeek => {
            object_receiver("SeekableStream")?;
            if arguments.len() != 2 || argument_type(1) != &Type::Int || result != Some(&Type::Void)
            {
                return Err("invalid stream `seek` signature".to_owned());
            }
            Ok(())
        }
        Builtin::StreamWriteAll => {
            object_receiver("WritableStream")?;
            if arguments.len() != 2
                || argument_type(1) != &Type::String
                || result != Some(&Type::Void)
            {
                return Err("invalid stream `writeAll` signature".to_owned());
            }
            Ok(())
        }
        Builtin::StreamClose => {
            object_receiver("Closeable")?;
            if arguments.len() != 1 || result != Some(&Type::Void) {
                return Err("invalid stream `close` signature".to_owned());
            }
            Ok(())
        }
        Builtin::ExceptionNew => {
            if !(arguments.len() == 1
                && argument_type(0) == &Type::String
                && matches!(result, Some(Type::Object(_))))
            {
                return Err("invalid native exception constructor signature".to_owned());
            }
            Ok(())
        }
        Builtin::ExceptionConstruct => {
            object_receiver("Throwable")?;
            let previous = Type::Union(vec![Type::Object("Throwable".to_owned()), Type::Null]);
            if arguments.len() != 4
                || argument_type(1) != &Type::String
                || argument_type(2) != &Type::Int
                || !type_accepts(program, &previous, argument_type(3))
                || result != Some(&Type::Void)
            {
                return Err("invalid native exception constructor signature".to_owned());
            }
            Ok(())
        }
        Builtin::ExceptionGetMessage | Builtin::ExceptionGetTarget => {
            object_receiver(if builtin == Builtin::ExceptionGetTarget {
                "OpenStreamException"
            } else {
                "Throwable"
            })?;
            if arguments.len() != 1 || result != Some(&Type::String) {
                return Err("invalid native exception string method signature".to_owned());
            }
            Ok(())
        }
        Builtin::ExceptionGetSystemCode => {
            object_receiver("OpenStreamException")?;
            if arguments.len() != 1 || result != Some(&Type::Int) {
                return Err("invalid native exception system-code signature".to_owned());
            }
            Ok(())
        }
        Builtin::ExceptionGetCode => {
            object_receiver("Throwable")?;
            if arguments.len() != 1 || result != Some(&Type::Int) {
                return Err("invalid native exception code signature".to_owned());
            }
            Ok(())
        }
        Builtin::ExceptionGetPrevious => {
            object_receiver("Throwable")?;
            if arguments.len() != 1
                || result
                    != Some(&Type::Union(vec![
                        Type::Object("Throwable".to_owned()),
                        Type::Null,
                    ]))
            {
                return Err("invalid native exception previous signature".to_owned());
            }
            Ok(())
        }
        Builtin::ExceptionGetSuppressed => {
            object_receiver("Throwable")?;
            if arguments.len() != 1
                || result
                    != Some(&Type::Vector(Box::new(Type::Object(
                        "Throwable".to_owned(),
                    ))))
            {
                return Err("invalid native exception suppressed-list signature".to_owned());
            }
            Ok(())
        }
    }
}

fn verify_terminator(
    program: &Program,
    function: &Function,
    block: &Block,
) -> Result<(), VerificationError> {
    match block.terminator {
        Terminator::Jump(target) => check_block(function, target, block.id, None),
        Terminator::Branch {
            condition,
            then_block,
            else_block,
        } => {
            check_register(function, condition, block.id, None)?;
            if function.register_types[condition.0 as usize] != Type::Bool {
                return Err(block_error(
                    function.id,
                    block.id,
                    "branch condition is not bool",
                ));
            }
            check_block(function, then_block, block.id, None)?;
            check_block(function, else_block, block.id, None)
        }
        Terminator::Return(value) => match value {
            Some(value) => {
                check_register(function, value, block.id, None)?;
                if !type_accepts(
                    program,
                    &function.return_type,
                    &function.register_types[value.0 as usize],
                ) {
                    return Err(block_error(
                        function.id,
                        block.id,
                        "returned register type does not match function return type",
                    ));
                }
                Ok(())
            }
            None if function.return_type == Type::Void => Ok(()),
            None => Err(block_error(
                function.id,
                block.id,
                "non-void function returns without a value",
            )),
        },
        Terminator::Throw(value) => {
            check_register(function, value, block.id, None)?;
            let throwable = match &function.register_types[value.0 as usize] {
                Type::Object(name) => is_instance_of_name(program, name, "Throwable"),
                _ => false,
            };
            if !throwable {
                return Err(block_error(
                    function.id,
                    block.id,
                    "thrown register is not a Throwable",
                ));
            }
            Ok(())
        }
        Terminator::Unreachable => Ok(()),
    }
}

fn check_local(
    function: &Function,
    local: LocalId,
    block: BlockId,
    instruction: Option<usize>,
) -> Result<(), VerificationError> {
    if local.0 as usize >= function.local_types.len() {
        Err(VerificationError {
            function: Some(function.id),
            block: Some(block),
            instruction,
            message: format!("local {} is out of bounds", local.0),
        })
    } else {
        Ok(())
    }
}

fn check_register(
    function: &Function,
    register: Register,
    block: BlockId,
    instruction: Option<usize>,
) -> Result<(), VerificationError> {
    if register.0 as usize >= function.register_types.len() {
        Err(VerificationError {
            function: Some(function.id),
            block: Some(block),
            instruction,
            message: format!("register {} is out of bounds", register.0),
        })
    } else {
        Ok(())
    }
}

fn check_block(
    function: &Function,
    target: BlockId,
    block: BlockId,
    instruction: Option<usize>,
) -> Result<(), VerificationError> {
    if target.0 as usize >= function.blocks.len() {
        Err(VerificationError {
            function: Some(function.id),
            block: Some(block),
            instruction,
            message: format!("target block {} is out of bounds", target.0),
        })
    } else {
        Ok(())
    }
}

fn type_accepts(program: &Program, expected: &Type, actual: &Type) -> bool {
    expected == &Type::Mixed
        || actual == &Type::Never
        || expected == actual
        || matches!(
            actual,
            Type::Union(members)
                if members
                    .iter()
                    .all(|member| type_accepts(program, expected, member))
        )
        || matches!(
            expected,
            Type::Union(members)
                if members
                    .iter()
                    .any(|member| type_accepts(program, member, actual))
        )
        || matches!(
            (expected, actual),
            (Type::Object(expected), Type::Object(actual))
                if is_instance_of_name(program, actual, expected)
        )
}

fn is_instance_of_name(program: &Program, actual: &str, expected: &str) -> bool {
    let Some(actual) = class_by_name(program, actual) else {
        return false;
    };
    let Some(expected) = class_by_name(program, expected) else {
        return false;
    };
    is_instance_of_id(program, actual.id, expected.id)
}

fn is_instance_of_id(program: &Program, mut actual: ClassId, expected: ClassId) -> bool {
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

fn constant_type(constant: &Constant) -> Type {
    match constant {
        Constant::Integer(_) => Type::Int,
        Constant::Float(_) => Type::Float,
        Constant::Bool(_) => Type::Bool,
        Constant::Null => Type::Null,
        Constant::String(_) => Type::String,
    }
}

fn global_error(message: impl Into<String>) -> VerificationError {
    VerificationError {
        function: None,
        block: None,
        instruction: None,
        message: message.into(),
    }
}

fn function_error(function: FunctionId, message: impl Into<String>) -> VerificationError {
    VerificationError {
        function: Some(function),
        block: None,
        instruction: None,
        message: message.into(),
    }
}

fn block_error(
    function: FunctionId,
    block: BlockId,
    message: impl Into<String>,
) -> VerificationError {
    VerificationError {
        function: Some(function),
        block: Some(block),
        instruction: None,
        message: message.into(),
    }
}

fn instruction_error(
    function: FunctionId,
    block: BlockId,
    instruction: usize,
    message: impl Into<String>,
) -> VerificationError {
    VerificationError {
        function: Some(function),
        block: Some(block),
        instruction: Some(instruction),
        message: message.into(),
    }
}

impl fmt::Display for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "THP bytecode v{} entry=#{}",
            self.schema_version, self.entry.0
        )?;
        for function in &self.functions {
            writeln!(
                formatter,
                "function #{} {} locals={} registers={} -> {}",
                function.id.0,
                function.name,
                function.local_types.len(),
                function.register_types.len(),
                function.return_type
            )?;
            for block in &function.blocks {
                writeln!(formatter, "  block {}:", block.id.0)?;
                for instruction in &block.instructions {
                    match instruction.destination {
                        Some(register) => write!(formatter, "    r{} = ", register.0)?,
                        None => formatter.write_str("    ")?,
                    }
                    writeln!(formatter, "{:?}", instruction.kind)?;
                }
                writeln!(formatter, "    {:?}", block.terminator)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use thp_diagnostics::SourceFile;
    use thp_hir::lower as lower_hir;
    use thp_mir::lower as lower_mir;
    use thp_syntax::parse;

    use super::{BYTECODE_SCHEMA_VERSION, decode, encode, lower, verify};

    fn compile(source: &str) -> super::Program {
        let source = SourceFile::new("test.thp", source);
        let parsed = parse(&source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let hir = lower_hir(&parsed.program);
        assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);
        lower(&lower_mir(&hir.module))
    }

    #[test]
    fn verifies_well_typed_program() {
        let program = compile("<?thp\n$value: int = 2 + 3; echo $value;");
        assert_eq!(program.schema_version, BYTECODE_SCHEMA_VERSION);
        verify(&program).unwrap();
    }

    #[test]
    fn rejects_bad_branch_target() {
        let mut program = compile("<?thp\nif (true) { echo \"yes\"; }");
        program.functions[0].blocks[0].terminator = super::Terminator::Jump(thp_mir::BlockId(999));
        assert!(verify(&program).is_err());
    }

    #[test]
    fn rejects_non_printable_bytecode_operand() {
        let mut program = compile("<?thp\necho \"1\";");
        let instructions = &mut program.functions[0].blocks[0].instructions;
        let register = match instructions[1].kind {
            super::InstructionKind::Print(register) => register,
            ref instruction => panic!("expected print, found {instruction:?}"),
        };
        instructions[0].kind = super::InstructionKind::Constant(super::Constant::Null);
        instructions[0].ty = Some(thp_hir::Type::Null);
        program.functions[0].register_types[register.0 as usize] = thp_hir::Type::Null;
        assert!(
            verify(&program)
                .unwrap_err()
                .to_string()
                .contains("print operand must be an output scalar")
        );
    }

    #[test]
    fn rejects_non_printable_concatenation_operand() {
        let mut program = compile("<?thp\necho \"a\" . \"b\";");
        let instructions = &mut program.functions[0].blocks[0].instructions;
        instructions[0].kind = super::InstructionKind::Constant(super::Constant::Null);
        instructions[0].ty = Some(thp_hir::Type::Null);
        program.functions[0].register_types[0] = thp_hir::Type::Null;
        assert!(
            verify(&program)
                .unwrap_err()
                .to_string()
                .contains("concatenation operands must be output scalars")
        );
    }

    #[test]
    fn bytecode_round_trips() {
        let program = compile(
            "<?thp\nfunction add(int $a, int $b): int { return $a + $b; } echo add(2, 3) . \"\";",
        );
        let encoded = encode(&program);
        let decoded = decode(&encoded).unwrap();
        verify(&decoded).unwrap();
        assert_eq!(decoded.instruction_count(), program.instruction_count());
        assert_eq!(decoded.to_string(), program.to_string());
    }

    #[test]
    fn object_metadata_and_binary_strings_round_trip() {
        let program = compile(
            r#"<?thp
class Box {
    public string $value;
    public function __construct(string $value) { $this->value = $value; }
}
$box = new Box("\x00\xff");
"#,
        );
        let decoded = decode(&encode(&program)).unwrap();
        verify(&decoded).unwrap();
        assert!(decoded.classes.iter().any(|class| class.name == "Box"));
    }

    #[test]
    fn exception_handlers_round_trip() {
        let program = compile(
            r#"<?thp
class Problem extends Exception {}
$previous: ?Throwable = null;
try {
    throw new Problem("problem", 0, $previous);
} catch (Problem $error) {
    echo "caught";
}
"#,
        );
        let decoded = decode(&encode(&program)).unwrap();
        verify(&decoded).unwrap();
        assert_eq!(decoded.functions[0].exception_handlers.len(), 1);
        let problem = decoded
            .classes
            .iter()
            .find(|class| class.name == "Problem")
            .unwrap()
            .id;
        assert_eq!(
            decoded.functions[0].exception_handlers[0].catches[0].class,
            Some(problem)
        );
    }

    #[test]
    fn essential_control_flow_instructions_round_trip() {
        let program = compile(
            r"<?thp
$values: vector<int> = [1];
foreach ($values as $index => $value) {
    $values[$index] = match ($value) { 1 => 2 };
}
",
        );
        let decoded = decode(&encode(&program)).unwrap();
        verify(&decoded).unwrap();
        let instructions = decoded.functions[0]
            .blocks
            .iter()
            .flat_map(|block| &block.instructions)
            .map(|instruction| &instruction.kind)
            .collect::<Vec<_>>();
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, super::InstructionKind::CollectionLen(_)))
        );
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, super::InstructionKind::SetIndex { .. }))
        );
        assert!(
            instructions
                .iter()
                .any(|kind| matches!(kind, super::InstructionKind::RaiseUnhandledMatch(_)))
        );
    }

    #[test]
    fn rejects_unsupported_schema_and_forged_object_descriptors() {
        let mut unsupported = compile("<?thp\necho \"1\";");
        unsupported.schema_version = 0;
        assert!(verify(&unsupported).is_err());
        assert!(decode(&encode(&unsupported)).is_err());

        let mut incomplete = compile(
            r"<?thp
abstract class Work {
    abstract public function run(): void;
}
",
        );
        let work = incomplete
            .classes
            .iter_mut()
            .find(|class| class.name == "Work")
            .unwrap();
        work.abstract_class = false;
        assert!(
            verify(&incomplete)
                .unwrap_err()
                .to_string()
                .contains("incomplete dispatch table")
        );

        let mut invalid_slot = compile(
            r"<?thp
class Work {
    public function run(): void {}
}
",
        );
        let work = invalid_slot
            .classes
            .iter_mut()
            .find(|class| class.name == "Work")
            .unwrap();
        work.methods[0].slot = thp_hir::MethodSlot(u32::MAX);
        assert!(verify(&invalid_slot).is_err());
    }

    #[test]
    fn rejects_forged_private_property_access() {
        let mut program = compile(
            r#"<?thp
class Box {
    public int $value = 1;
}
$box = new Box();
echo $box->value . "";
"#,
        );
        let class = program
            .classes
            .iter_mut()
            .find(|class| class.name == "Box")
            .unwrap();
        class.properties[0].visibility = thp_syntax::Visibility::Private;
        let error = verify(&program).unwrap_err();
        assert!(error.to_string().contains("property load violates"));
    }

    #[test]
    fn rejects_forged_handler_and_catch_ordering() {
        let mut regions = compile(
            r"<?thp
try {
    try {
        throw new Exception();
    } catch (Exception $inner) {}
} catch (Throwable $outer) {}
",
        );
        assert_eq!(regions.functions[0].exception_handlers.len(), 2);
        regions.functions[0].exception_handlers.swap(0, 1);
        assert!(
            verify(&regions)
                .unwrap_err()
                .to_string()
                .contains("innermost first")
        );

        let mut catches = compile(
            r"<?thp
try {
    throw new Exception();
} catch (Exception $specific) {
} catch (Throwable $general) {}
",
        );
        catches.functions[0].exception_handlers[0]
            .catches
            .swap(0, 1);
        assert!(
            verify(&catches)
                .unwrap_err()
                .to_string()
                .contains("subsumed")
        );
    }
}
