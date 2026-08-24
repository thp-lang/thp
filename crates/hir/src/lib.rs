//! Name resolution, static typing, and THP's typed high-level IR.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

use thp_diagnostics::{Diagnostic, Span};
use thp_syntax::{
    Argument, BinaryOp, Block, Expr, ExprKind, ForClause, ForClauseKind, FunctionDecl, MatchArm,
    MethodDecl, Program, ScopeTarget, Stmt, StmtKind, TraitAdaptation, TraitUse, TypeSyntax,
    TypeSyntaxKind, UnaryOp, Visibility,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Null,
    Void,
    Never,
    Mixed,
    Vector(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Union(Vec<Type>),
    Object(String),
}

impl Type {
    pub fn representation(&self) -> Representation {
        match self {
            Self::Int => Representation::I64,
            Self::Float => Representation::F64,
            Self::Bool => Representation::Bool,
            Self::String => Representation::Bytes,
            Self::Vector(_) | Self::Map(_, _) | Self::Object(_) => Representation::Reference,
            Self::Null | Self::Void | Self::Never | Self::Mixed | Self::Union(_) => {
                Representation::Value
            }
        }
    }

    pub fn accepts(&self, actual: &Self) -> bool {
        self == &Self::Mixed
            || actual == &Self::Never
            || self == actual
            || matches!(
                actual,
                Self::Union(members) if members.iter().all(|member| self.accepts(member))
            )
            || matches!(self, Self::Union(members) if members.iter().any(|member| member.accepts(actual)))
    }

    fn without_null(&self) -> Option<Self> {
        match self {
            Self::Union(members) => {
                let members = members
                    .iter()
                    .filter(|member| **member != Self::Null)
                    .cloned()
                    .collect::<Vec<_>>();
                match members.as_slice() {
                    [] => None,
                    [member] => Some(member.clone()),
                    _ => Some(Self::Union(members)),
                }
            }
            Self::Null => None,
            other => Some(other.clone()),
        }
    }

    pub fn is_output_scalar(&self) -> bool {
        matches!(
            self,
            Self::Int | Self::Float | Self::Bool | Self::String | Self::Never
        ) || matches!(self, Self::Union(members) if !members.is_empty() && members.iter().all(Self::is_output_scalar))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => formatter.write_str("int"),
            Self::Float => formatter.write_str("float"),
            Self::Bool => formatter.write_str("bool"),
            Self::String => formatter.write_str("string"),
            Self::Null => formatter.write_str("null"),
            Self::Void => formatter.write_str("void"),
            Self::Never => formatter.write_str("never"),
            Self::Mixed => formatter.write_str("mixed"),
            Self::Vector(element) => write!(formatter, "vector<{element}>"),
            Self::Map(key, value) => write!(formatter, "map<{key}, {value}>"),
            Self::Object(name) => formatter.write_str(name),
            Self::Union(members) => {
                for (index, member) in members.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str("|")?;
                    }
                    write!(formatter, "{member}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Representation {
    I64,
    F64,
    Bool,
    Bytes,
    Reference,
    Value,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FunctionId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocalId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClassId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PropertyId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MethodSlot(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NominalKind {
    Class,
    Interface,
    Trait,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Builtin {
    Count,
    VarDump,
    MemoryStreamOpen,
    TempStreamOpen,
    StreamsOpen,
    FilesOpenRead,
    StreamTell,
    StreamRead,
    StreamReadAll,
    StreamEof,
    StreamSeek,
    StreamWriteAll,
    StreamClose,
    StreamIsClosed,
    ExceptionNew,
    ExceptionConstruct,
    ExceptionGetMessage,
    ExceptionGetCode,
    ExceptionGetPrevious,
    ExceptionGetTarget,
    ExceptionGetSystemCode,
    ExceptionGetSuppressed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Callee {
    Function(FunctionId),
    Builtin(Builtin),
}

#[derive(Clone, Debug)]
pub struct Module {
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
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub properties: Vec<Property>,
    pub methods: Vec<Method>,
    pub method_slots: Vec<MethodSlot>,
    pub dispatch: Vec<Option<Callee>>,
    pub constructor: Option<Callee>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Property {
    pub id: PropertyId,
    pub name: String,
    pub ty: Type,
    pub visibility: Visibility,
    pub declaring_class: ClassId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub name: String,
    pub callee: Option<Callee>,
    pub slot: MethodSlot,
    pub declaring_class: ClassId,
    pub visibility: Visibility,
    pub static_method: bool,
    pub abstract_method: bool,
    pub final_method: bool,
    pub parameter_types: Vec<Type>,
    pub return_type: Type,
    pub span: Span,
}

impl Module {
    pub fn function(&self, id: FunctionId) -> &Function {
        &self.functions[id.0 as usize]
    }

    pub fn expression_count(&self) -> usize {
        self.functions
            .iter()
            .map(|function| count_expressions(&function.body))
            .sum()
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: FunctionId,
    pub name: String,
    pub parameters: Vec<LocalId>,
    pub locals: Vec<Local>,
    pub return_type: Type,
    pub owner: Option<ClassId>,
    pub static_method: bool,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Local {
    pub id: LocalId,
    pub name: String,
    pub ty: Type,
    pub span: Span,
    pub parameter: bool,
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum StatementKind {
    Assign {
        local: LocalId,
        value: TypedExpr,
        declares: bool,
    },
    Echo(TypedExpr),
    Return(Option<TypedExpr>),
    If {
        branches: Vec<(TypedExpr, Vec<Statement>)>,
        otherwise: Vec<Statement>,
    },
    While {
        condition: TypedExpr,
        body: Vec<Statement>,
    },
    For {
        initializers: Vec<LoopClause>,
        conditions: Vec<LoopClause>,
        updates: Vec<LoopClause>,
        body: Vec<Statement>,
    },
    Foreach {
        source: TypedExpr,
        key: Option<LocalId>,
        value: LocalId,
        key_type: Type,
        value_type: Type,
        body: Vec<Statement>,
    },
    Break,
    Continue,
    SetProperty {
        object: TypedExpr,
        property: PropertyId,
        value: TypedExpr,
    },
    SetIndex {
        root: LocalId,
        collection_types: Vec<Type>,
        indices: Vec<TypedExpr>,
        value: TypedExpr,
    },
    Throw(TypedExpr),
    Try {
        body: Vec<Statement>,
        catches: Vec<Catch>,
        finally: Option<Vec<Statement>>,
    },
    Using {
        local: LocalId,
        value: TypedExpr,
        close: MethodSlot,
        body: Vec<Statement>,
    },
    Block(Vec<Statement>),
    Expression(TypedExpr),
}

#[derive(Clone, Debug)]
pub struct LoopClause {
    pub kind: LoopClauseKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum LoopClauseKind {
    Assign {
        local: LocalId,
        value: TypedExpr,
    },
    SetProperty {
        object: TypedExpr,
        property: PropertyId,
        value: TypedExpr,
    },
    SetIndex {
        root: LocalId,
        collection_types: Vec<Type>,
        indices: Vec<TypedExpr>,
        value: TypedExpr,
    },
    Expression(TypedExpr),
}

#[derive(Clone, Debug)]
pub struct Catch {
    pub class: ClassId,
    pub local: LocalId,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TypedExprKind {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,
    String(Vec<u8>),
    Local(LocalId),
    Vector(Vec<TypedExpr>),
    Map(Vec<(TypedExpr, TypedExpr)>),
    Unary {
        op: UnaryOp,
        operand: Box<TypedExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
    Call {
        callee: Callee,
        arguments: BoundArguments,
    },
    DirectMethod {
        callee: Callee,
        receiver: Option<Box<TypedExpr>>,
        arguments: BoundArguments,
        called_class: CalledClass,
    },
    VirtualMethod {
        receiver: Box<TypedExpr>,
        slot: MethodSlot,
        arguments: BoundArguments,
    },
    LateStaticMethod {
        receiver: Option<Box<TypedExpr>>,
        slot: MethodSlot,
        arguments: BoundArguments,
    },
    Index {
        collection: Box<TypedExpr>,
        index: Box<TypedExpr>,
    },
    New {
        class: ClassId,
        constructor: Option<Callee>,
        initializers: Vec<(PropertyId, TypedExpr)>,
        arguments: BoundArguments,
    },
    Property {
        object: Box<TypedExpr>,
        property: PropertyId,
    },
    InstanceOf {
        value: Box<TypedExpr>,
        class: ClassId,
    },
    Match {
        subject: Box<TypedExpr>,
        arms: Vec<TypedMatchArm>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalledClass {
    Explicit(ClassId),
    Forwarded,
    Receiver,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgumentTarget {
    Parameter(usize),
    Variadic,
}

#[derive(Clone, Debug)]
pub struct BoundArgument {
    pub target: ArgumentTarget,
    pub value: TypedExpr,
}

#[derive(Clone, Debug)]
pub struct BoundArguments {
    pub explicit: Vec<BoundArgument>,
    pub defaults: Vec<BoundArgument>,
    pub parameter_count: usize,
    pub variadic_parameter: Option<usize>,
    pub variadic_type: Option<Type>,
}

#[derive(Clone, Debug)]
pub struct TypedMatchArm {
    pub conditions: Vec<TypedExpr>,
    pub value: TypedExpr,
    pub default: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct LowerOutput {
    pub module: Module,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
struct Signature {
    id: FunctionId,
    parameters: Vec<ParameterSignature>,
    return_type: Type,
    span: Span,
}

#[derive(Clone, Debug, PartialEq)]
struct ParameterSignature {
    name: String,
    ty: Type,
    default: Option<Expr>,
    variadic: bool,
    span: Span,
}

#[derive(Clone, Debug)]
struct MethodSignature {
    signature: Signature,
    callee: Option<Callee>,
    slot: MethodSlot,
    declaring_class: ClassId,
    visibility: Visibility,
    static_method: bool,
    abstract_method: bool,
    final_method: bool,
    source: Option<MethodDecl>,
    origin_trait: Option<String>,
    span: Span,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
struct ClassSignature {
    id: ClassId,
    name: String,
    kind: NominalKind,
    abstract_class: bool,
    final_class: bool,
    parent: Option<String>,
    interfaces: Vec<String>,
    trait_uses: Vec<TraitUse>,
    declared_properties: Vec<Property>,
    declared_property_initializers: Vec<Option<Expr>>,
    declared_methods: BTreeMap<String, MethodSignature>,
    properties: Vec<Property>,
    property_initializers: Vec<Option<Expr>>,
    methods: BTreeMap<String, MethodSignature>,
    constructor: Option<Callee>,
    native: bool,
    linked: bool,
    span: Span,
}

#[derive(Clone, Debug)]
struct PendingMethod {
    id: FunctionId,
    owner: ClassId,
    declaration: MethodDecl,
    signature: Signature,
    span: Span,
}

/// Resolves names and produces typed HIR. A module may be inspected when
/// diagnostics exist, but must not be lowered to executable code.
pub fn lower(program: &Program) -> LowerOutput {
    TypeChecker::new(program).lower(program)
}

struct TypeChecker {
    diagnostics: Vec<Diagnostic>,
    signatures: BTreeMap<String, Signature>,
    classes: BTreeMap<String, ClassSignature>,
    functions: Vec<Option<Function>>,
    pending_methods: Vec<PendingMethod>,
    method_slots: BTreeMap<String, MethodSlot>,
}

impl TypeChecker {
    fn new(_program: &Program) -> Self {
        Self {
            diagnostics: Vec::new(),
            signatures: BTreeMap::new(),
            classes: native_nominals(),
            functions: vec![None],
            pending_methods: Vec::new(),
            method_slots: BTreeMap::new(),
        }
    }

    fn lower(mut self, program: &Program) -> LowerOutput {
        self.collect_nominal_names(program);
        self.collect_signatures(program);
        self.validate_nominal_relationships();
        self.assign_method_slots();
        self.link_nominals();
        let top_level = program
            .statements
            .iter()
            .filter(|statement| {
                !matches!(
                    statement.kind,
                    StmtKind::Function(_)
                        | StmtKind::Class(_)
                        | StmtKind::Interface(_)
                        | StmtKind::Trait(_)
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let entry = FunctionId(0);
        let mut main = FunctionChecker::new(
            &self.signatures,
            &self.classes,
            &mut self.diagnostics,
            entry,
            "<main>",
            Type::Void,
            program.span,
            None,
            false,
        );
        let body = main.lower_block(&top_level);
        self.functions[0] = Some(main.finish(body));

        for statement in &program.statements {
            let StmtKind::Function(declaration) = &statement.kind else {
                continue;
            };
            let Some(signature) = self.signatures.get(&declaration.name).cloned() else {
                continue;
            };
            let mut checker = FunctionChecker::new(
                &self.signatures,
                &self.classes,
                &mut self.diagnostics,
                signature.id,
                &declaration.name,
                signature.return_type.clone(),
                statement.span,
                None,
                false,
            );
            checker.declare_parameters(declaration, &signature.parameters);
            let body = checker.lower_block(&declaration.body);
            let index = signature.id.0 as usize;
            self.functions[index] = Some(checker.finish(body));
        }

        for pending in self.pending_methods.clone() {
            let Some(class) = self
                .classes
                .values()
                .find(|class| class.id == pending.owner)
                .cloned()
            else {
                continue;
            };
            let mut checker = FunctionChecker::new(
                &self.signatures,
                &self.classes,
                &mut self.diagnostics,
                pending.id,
                &format!("{}::{}", class.name, pending.declaration.function.name),
                pending.signature.return_type.clone(),
                pending.span,
                Some(class.id),
                pending.declaration.static_method,
            );
            if !pending.declaration.static_method {
                checker.declare_receiver(&class);
            }
            checker
                .declare_parameters(&pending.declaration.function, &pending.signature.parameters);
            let body = checker.lower_block(&pending.declaration.function.body);
            self.functions[pending.id.0 as usize] = Some(checker.finish(body));
        }

        let functions = self
            .functions
            .into_iter()
            .enumerate()
            .map(|(index, function)| {
                function.unwrap_or_else(|| Function {
                    id: FunctionId(
                        u32::try_from(index).expect("function count is limited to u32::MAX"),
                    ),
                    name: format!("<invalid-{index}>"),
                    parameters: Vec::new(),
                    locals: Vec::new(),
                    return_type: Type::Never,
                    owner: None,
                    static_method: false,
                    body: Vec::new(),
                    span: program.span,
                })
            })
            .collect();
        let mut classes = self
            .classes
            .into_values()
            .map(|class| Class {
                id: class.id,
                name: class.name,
                kind: class.kind,
                abstract_class: class.abstract_class,
                final_class: class.final_class,
                parent: class.parent,
                interfaces: class.interfaces,
                properties: class.properties,
                methods: class
                    .methods
                    .clone()
                    .into_iter()
                    .map(|(name, method)| Method {
                        name,
                        callee: method.callee,
                        slot: method.slot,
                        declaring_class: method.declaring_class,
                        visibility: method.visibility,
                        static_method: method.static_method,
                        abstract_method: method.abstract_method,
                        final_method: method.final_method,
                        parameter_types: method
                            .signature
                            .parameters
                            .iter()
                            .map(|parameter| parameter.ty.clone())
                            .collect(),
                        return_type: method.signature.return_type,
                        span: method.span,
                    })
                    .collect(),
                method_slots: class.methods.values().map(|method| method.slot).collect(),
                dispatch: {
                    let mut dispatch = vec![None; self.method_slots.len()];
                    for method in class.methods.values() {
                        dispatch[method.slot.0 as usize] = method.callee;
                    }
                    dispatch
                },
                constructor: class.constructor,
                span: class.span,
            })
            .collect::<Vec<_>>();
        classes.sort_by_key(|class| class.id);
        LowerOutput {
            module: Module {
                functions,
                classes,
                entry,
            },
            diagnostics: self.diagnostics,
        }
    }

    fn collect_nominal_names(&mut self, program: &Program) {
        for statement in &program.statements {
            let (name, name_span, kind, abstract_class, final_class, parent, interfaces, uses) =
                match &statement.kind {
                    StmtKind::Class(declaration) => (
                        &declaration.name,
                        declaration.name_span,
                        NominalKind::Class,
                        declaration.abstract_class,
                        declaration.final_class,
                        declaration
                            .parent
                            .as_ref()
                            .map(|parent| parent.name.clone()),
                        declaration
                            .interfaces
                            .iter()
                            .map(|interface| interface.name.clone())
                            .collect(),
                        declaration.trait_uses.clone(),
                    ),
                    StmtKind::Interface(declaration) => (
                        &declaration.name,
                        declaration.name_span,
                        NominalKind::Interface,
                        true,
                        false,
                        declaration
                            .parent
                            .as_ref()
                            .map(|parent| parent.name.clone()),
                        Vec::new(),
                        Vec::new(),
                    ),
                    StmtKind::Trait(declaration) => (
                        &declaration.name,
                        declaration.name_span,
                        NominalKind::Trait,
                        true,
                        false,
                        None,
                        Vec::new(),
                        declaration.trait_uses.clone(),
                    ),
                    _ => continue,
                };
            if let Some(previous) = self.classes.get(name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "name_resolution",
                        "N0004",
                        name_span,
                        format!("nominal type `{name}` is declared more than once"),
                    )
                    .with_label(previous.span, "first declaration is here"),
                );
                continue;
            }
            let id = ClassId(
                u32::try_from(self.classes.len()).expect("class count is limited to u32::MAX"),
            );
            self.classes.insert(
                name.clone(),
                ClassSignature {
                    id,
                    name: name.clone(),
                    kind,
                    abstract_class,
                    final_class,
                    parent,
                    interfaces,
                    trait_uses: uses,
                    declared_properties: Vec::new(),
                    declared_property_initializers: Vec::new(),
                    declared_methods: BTreeMap::new(),
                    properties: Vec::new(),
                    property_initializers: Vec::new(),
                    methods: BTreeMap::new(),
                    constructor: None,
                    native: false,
                    linked: false,
                    span: statement.span,
                },
            );
        }
    }

    fn validate_nominal_relationships(&mut self) {
        let classes = self.classes.values().cloned().collect::<Vec<_>>();
        for class in &classes {
            if let Some(parent_name) = &class.parent {
                match self.classes.get(parent_name) {
                    None => self.diagnostics.push(Diagnostic::error(
                        "name_resolution",
                        "N0007",
                        class.span,
                        format!("unknown parent class `{parent_name}`"),
                    )),
                    Some(parent) if class.kind != parent.kind => {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0012",
                            class.span,
                            format!(
                                "{} `{}` cannot extend {} `{parent_name}`",
                                nominal_kind_name(class.kind),
                                class.name,
                                nominal_kind_name(parent.kind)
                            ),
                        ));
                    }
                    Some(parent) if class.kind == NominalKind::Class && parent.final_class => {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0010",
                            class.span,
                            format!("final class `{parent_name}` cannot be extended"),
                        ));
                    }
                    Some(_) => {}
                }
            }
            for interface in &class.interfaces {
                match self.classes.get(interface) {
                    None => self.diagnostics.push(Diagnostic::error(
                        "name_resolution",
                        "N0008",
                        class.span,
                        format!("unknown implemented interface `{interface}`"),
                    )),
                    Some(target) if target.kind != NominalKind::Interface => {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0013",
                            class.span,
                            format!("`{interface}` is not an interface"),
                        ));
                    }
                    Some(_) => {}
                }
            }
            for trait_use in &class.trait_uses {
                for used in &trait_use.traits {
                    match self.classes.get(&used.name) {
                        None => self.diagnostics.push(Diagnostic::error(
                            "name_resolution",
                            "N0009",
                            used.span,
                            format!("unknown trait `{}`", used.name),
                        )),
                        Some(target) if target.kind != NominalKind::Trait => {
                            self.diagnostics.push(Diagnostic::error(
                                "typing",
                                "T0015",
                                used.span,
                                format!("`{}` is not a trait", used.name),
                            ));
                        }
                        Some(_) => {}
                    }
                }
            }
        }
    }

    fn collect_signatures(&mut self, program: &Program) {
        for statement in &program.statements {
            let StmtKind::Function(declaration) = &statement.kind else {
                continue;
            };
            if let Some(previous) = self.signatures.get(&declaration.name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "name_resolution",
                        "N0001",
                        declaration.name_span,
                        format!("function `{}` is declared more than once", declaration.name),
                    )
                    .with_label(previous.span, "first declaration is here"),
                );
                continue;
            }
            let parameters = resolve_parameters(declaration, &self.classes, &mut self.diagnostics);
            let return_type = resolve_type(
                &declaration.return_type,
                &self.classes,
                &mut self.diagnostics,
            )
            .unwrap_or(Type::Mixed);
            let id = FunctionId(
                u32::try_from(self.functions.len()).expect("function count is limited to u32::MAX"),
            );
            self.functions.push(None);
            self.signatures.insert(
                declaration.name.clone(),
                Signature {
                    id,
                    parameters,
                    return_type,
                    span: declaration.name_span,
                },
            );
        }

        for statement in &program.statements {
            let (name, source_properties, source_methods): (
                &str,
                &[thp_syntax::PropertyDecl],
                &[MethodDecl],
            ) = match &statement.kind {
                StmtKind::Class(declaration) => (
                    declaration.name.as_str(),
                    &declaration.properties,
                    &declaration.methods,
                ),
                StmtKind::Interface(declaration) => {
                    (declaration.name.as_str(), &[], &declaration.methods)
                }
                StmtKind::Trait(declaration) => (
                    declaration.name.as_str(),
                    &declaration.properties,
                    &declaration.methods,
                ),
                _ => continue,
            };
            let Some(existing) = self.classes.get(name).cloned() else {
                continue;
            };
            if existing.span != statement.span {
                continue;
            }
            let mut properties = Vec::new();
            let mut property_initializers = Vec::new();
            let mut property_names = BTreeMap::<String, Span>::new();
            for property in source_properties {
                if let Some(previous) = property_names.get(&property.name) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "name_resolution",
                            "N0005",
                            property.name_span,
                            format!("property `${}` is declared more than once", property.name),
                        )
                        .with_label(*previous, "first property is here"),
                    );
                    continue;
                }
                property_names.insert(property.name.clone(), property.name_span);
                let ty = resolve_type(&property.ty, &self.classes, &mut self.diagnostics)
                    .unwrap_or(Type::Mixed);
                properties.push(Property {
                    id: PropertyId(
                        u32::try_from(properties.len())
                            .expect("property count is limited to u32::MAX"),
                    ),
                    name: property.name.clone(),
                    ty,
                    visibility: property.visibility,
                    declaring_class: existing.id,
                    span: property.span,
                });
                property_initializers.push(property.initializer.clone());
            }
            let mut methods = BTreeMap::new();
            for method in source_methods {
                if let Some(previous) = methods.get(&method.function.name) {
                    let previous: &MethodSignature = previous;
                    self.diagnostics.push(
                        Diagnostic::error(
                            "name_resolution",
                            "N0006",
                            method.function.name_span,
                            format!(
                                "method `{}::{}` is declared more than once",
                                name, method.function.name
                            ),
                        )
                        .with_label(previous.span, "first method is here"),
                    );
                    continue;
                }
                let parameters =
                    resolve_parameters(&method.function, &self.classes, &mut self.diagnostics);
                let return_type = resolve_type(
                    &method.function.return_type,
                    &self.classes,
                    &mut self.diagnostics,
                )
                .unwrap_or(Type::Mixed);
                if method.abstract_method && method.visibility == Visibility::Private {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0016",
                        method.span,
                        "abstract methods cannot be private",
                    ));
                }
                if method.abstract_method && method.final_method {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0017",
                        method.span,
                        "abstract methods cannot be final",
                    ));
                }
                let concrete_user_method =
                    existing.kind == NominalKind::Class && !method.abstract_method;
                let id = if concrete_user_method {
                    self.allocate_function()
                } else {
                    FunctionId(u32::MAX)
                };
                let signature = Signature {
                    id,
                    parameters,
                    return_type,
                    span: method.function.name_span,
                };
                if method.function.name == "__construct" && method.static_method {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0007",
                        method.span,
                        "constructors cannot be static",
                    ));
                }
                methods.insert(
                    method.function.name.clone(),
                    MethodSignature {
                        signature,
                        callee: concrete_user_method.then_some(Callee::Function(id)),
                        slot: MethodSlot(u32::MAX),
                        declaring_class: existing.id,
                        visibility: method.visibility,
                        static_method: method.static_method,
                        abstract_method: method.abstract_method,
                        final_method: method.final_method,
                        source: Some(method.clone()),
                        origin_trait: (existing.kind == NominalKind::Trait)
                            .then(|| existing.name.clone()),
                        span: method.span,
                    },
                );
                if concrete_user_method {
                    self.pending_methods.push(PendingMethod {
                        id,
                        owner: existing.id,
                        declaration: method.clone(),
                        signature: methods[&method.function.name].signature.clone(),
                        span: method.span,
                    });
                }
            }
            let class = self
                .classes
                .get_mut(name)
                .expect("nominal type was predeclared");
            class.declared_properties = properties;
            class.declared_property_initializers = property_initializers;
            class.declared_methods = methods;
        }
    }

    fn allocate_function(&mut self) -> FunctionId {
        let id = FunctionId(
            u32::try_from(self.functions.len()).expect("function count is limited to u32::MAX"),
        );
        self.functions.push(None);
        id
    }

    fn assign_method_slots(&mut self) {
        let mut names = self
            .classes
            .values()
            .flat_map(|class| {
                class
                    .declared_methods
                    .keys()
                    .chain(class.methods.keys())
                    .cloned()
            })
            .collect::<BTreeSet<_>>();
        for class in self.classes.values() {
            for trait_use in &class.trait_uses {
                for adaptation in &trait_use.adaptations {
                    if let TraitAdaptation::Alias {
                        alias: Some(alias), ..
                    } = adaptation
                    {
                        names.insert(alias.name.clone());
                    }
                }
            }
        }
        self.method_slots = names
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                (
                    name,
                    MethodSlot(
                        u32::try_from(index).expect("method slot count is limited to u32::MAX"),
                    ),
                )
            })
            .collect();
        for class in self.classes.values_mut() {
            for (name, method) in class
                .declared_methods
                .iter_mut()
                .chain(class.methods.iter_mut())
            {
                method.slot = self.method_slots[name];
            }
        }
    }

    fn link_nominals(&mut self) {
        let names = self.classes.keys().cloned().collect::<Vec<_>>();
        for name in names {
            self.link_nominal(&name, &mut Vec::new());
        }
    }

    fn link_nominal(&mut self, name: &str, stack: &mut Vec<String>) {
        let Some(current) = self.classes.get(name).cloned() else {
            return;
        };
        if current.linked {
            return;
        }
        if stack.iter().any(|entry| entry == name) {
            self.diagnostics.push(Diagnostic::error(
                "typing",
                "T0011",
                current.span,
                format!("nominal dependency cycle involving `{name}`"),
            ));
            return;
        }
        stack.push(name.to_owned());
        if let Some(parent) = &current.parent {
            self.link_nominal(parent, stack);
        }
        for interface in &current.interfaces {
            self.link_nominal(interface, stack);
        }
        for trait_use in &current.trait_uses {
            for used in &trait_use.traits {
                self.link_nominal(&used.name, stack);
            }
        }

        let mut properties = Vec::new();
        let mut initializers = Vec::new();
        let mut methods: BTreeMap<String, MethodSignature> = BTreeMap::new();
        let mut interfaces = current.interfaces.clone();
        if let Some(parent_name) = &current.parent
            && let Some(parent) = self.classes.get(parent_name).filter(|parent| parent.linked)
        {
            properties.clone_from(&parent.properties);
            initializers.clone_from(&parent.property_initializers);
            methods = parent.methods.clone();
            if current.kind == NominalKind::Class {
                interfaces.extend(parent.interfaces.clone());
            } else if current.kind == NominalKind::Interface {
                interfaces.push(parent.name.clone());
                interfaces.extend(parent.interfaces.clone());
            }
        }

        if current.kind != NominalKind::Interface {
            let (trait_properties, trait_initializers, trait_methods) =
                self.compose_traits(&current);
            for (property, initializer) in trait_properties.into_iter().zip(trait_initializers) {
                if let Some(inherited) = properties
                    .iter()
                    .find(|inherited: &&Property| inherited.name == property.name)
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "typing",
                            "T0020",
                            property.span,
                            format!(
                                "trait property `${}` collides with an inherited property",
                                property.name
                            ),
                        )
                        .with_label(inherited.span, "inherited property is declared here"),
                    );
                } else {
                    properties.push(property);
                    initializers.push(initializer);
                }
            }
            for (method_name, method) in trait_methods {
                if let Some(parent) = methods.get(&method_name) {
                    self.validate_override(name, &method_name, &method, parent);
                }
                methods.insert(method_name, method);
            }
        }

        for (property, initializer) in current
            .declared_properties
            .iter()
            .cloned()
            .zip(current.declared_property_initializers.iter().cloned())
        {
            if let Some(previous) = properties
                .iter()
                .find(|previous| previous.name == property.name)
            {
                self.diagnostics.push(
                    Diagnostic::error(
                        "typing",
                        "T0021",
                        property.span,
                        format!(
                            "property `${}` cannot override an inherited or trait property",
                            property.name
                        ),
                    )
                    .with_label(previous.span, "previous property is declared here"),
                );
                continue;
            }
            let mut property = property;
            property.id = PropertyId(
                u32::try_from(properties.len()).expect("property count is limited to u32::MAX"),
            );
            properties.push(property);
            initializers.push(initializer);
        }
        for (name, method) in &current.declared_methods {
            if let Some(previous) = methods.get(name) {
                self.validate_override(&current.name, name, method, previous);
            }
            methods.insert(name.clone(), method.clone());
        }

        if current.kind == NominalKind::Class {
            let direct_interfaces = current.interfaces.clone();
            for interface_name in direct_interfaces {
                let Some(interface) = self.classes.get(&interface_name).cloned() else {
                    continue;
                };
                interfaces.push(interface.name.clone());
                interfaces.extend(interface.interfaces.clone());
                for (method_name, requirement) in interface.methods {
                    if let Some(existing) = methods.get(&method_name) {
                        if !method_contract_equal(existing, &requirement)
                            || existing.visibility != Visibility::Public
                        {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "typing",
                                    "T0022",
                                    existing.span,
                                    format!(
                                        "method `{}::{method_name}` does not satisfy interface `{}`",
                                        current.name, interface.name
                                    ),
                                )
                                .with_label(requirement.span, "required signature is here"),
                            );
                        }
                    } else {
                        methods.insert(method_name, requirement);
                    }
                }
            }
        } else if current.kind == NominalKind::Interface {
            for (name, requirement) in &current.declared_methods {
                if let Some(previous) = methods.get(name)
                    && !method_contract_equal(requirement, previous)
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "typing",
                            "T0023",
                            requirement.span,
                            format!("conflicting interface requirement for `{name}`"),
                        )
                        .with_label(previous.span, "inherited requirement is here"),
                    );
                }
                methods.insert(name.clone(), requirement.clone());
            }
        }

        interfaces.sort();
        interfaces.dedup();
        if interfaces.iter().any(|interface| interface == "Throwable") {
            let allowed = match current.kind {
                NominalKind::Interface => current.name == "Throwable",
                NominalKind::Class => {
                    current.native
                        || current.parent.as_deref().is_some_and(|parent| {
                            is_nominal_subtype(&self.classes, parent, "Exception")
                                || is_nominal_subtype(&self.classes, parent, "Error")
                        })
                }
                NominalKind::Trait => false,
            };
            if !allowed {
                self.diagnostics.push(Diagnostic::error(
                    "typing",
                    "T0014",
                    current.span,
                    "`Throwable` is sealed; only `Exception`, `Error`, and their descendants are throwable",
                ));
            }
        }
        for (index, property) in properties.iter_mut().enumerate() {
            property.id =
                PropertyId(u32::try_from(index).expect("property count is limited to u32::MAX"));
        }
        let unresolved = methods
            .values()
            .filter(|method| method.abstract_method || method.callee.is_none())
            .map(|method| method.signature.span)
            .collect::<Vec<_>>();
        if current.kind == NominalKind::Class && !current.abstract_class && !unresolved.is_empty() {
            let mut diagnostic = Diagnostic::error(
                "typing",
                "T0024",
                current.span,
                format!("concrete class `{}` has unresolved methods", current.name),
            );
            for span in unresolved {
                diagnostic = diagnostic.with_label(span, "unresolved requirement");
            }
            self.diagnostics.push(diagnostic);
        }
        let constructor = methods.get("__construct").and_then(|method| method.callee);
        let class = self.classes.get_mut(name).expect("nominal type exists");
        class.properties = properties;
        class.property_initializers = initializers;
        class.methods = methods;
        class.interfaces = interfaces;
        class.constructor = constructor;
        class.linked = true;
        stack.pop();
    }

    fn compose_traits(
        &mut self,
        consumer: &ClassSignature,
    ) -> (
        Vec<Property>,
        Vec<Option<Expr>>,
        BTreeMap<String, MethodSignature>,
    ) {
        let mut properties: Vec<Property> = Vec::new();
        let mut initializers: Vec<Option<Expr>> = Vec::new();
        let mut methods: BTreeMap<String, MethodSignature> = BTreeMap::new();
        for trait_use in &consumer.trait_uses {
            let mut candidates = BTreeMap::<String, Vec<(String, MethodSignature)>>::new();
            for used in &trait_use.traits {
                let Some(trait_type) = self
                    .classes
                    .get(&used.name)
                    .filter(|candidate| candidate.kind == NominalKind::Trait && candidate.linked)
                    .cloned()
                else {
                    continue;
                };
                for (property, initializer) in trait_type
                    .properties
                    .iter()
                    .cloned()
                    .zip(trait_type.property_initializers.iter().cloned())
                {
                    if let Some(index) = properties
                        .iter()
                        .position(|existing| existing.name == property.name)
                    {
                        if !property_contract_equal(
                            &properties[index],
                            initializers[index].as_ref(),
                            &property,
                            initializer.as_ref(),
                        ) {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "typing",
                                    "T0025",
                                    property.span,
                                    format!("conflicting trait property `${}`", property.name),
                                )
                                .with_label(properties[index].span, "first contribution is here"),
                            );
                        }
                    } else {
                        let mut property = property;
                        property.declaring_class = consumer.id;
                        properties.push(property);
                        initializers.push(initializer);
                    }
                }
                for (name, mut method) in trait_type.methods {
                    method.origin_trait = Some(used.name.clone());
                    candidates
                        .entry(name)
                        .or_default()
                        .push((used.name.clone(), method));
                }
            }
            let original_candidates = candidates.clone();
            for adaptation in &trait_use.adaptations {
                if let TraitAdaptation::InsteadOf {
                    trait_name,
                    method,
                    excluded,
                    span,
                } = adaptation
                {
                    let Some(entries) = candidates.get_mut(&method.name) else {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0026",
                            *span,
                            format!(
                                "unknown trait method `{}::{}`",
                                trait_name.name, method.name
                            ),
                        ));
                        continue;
                    };
                    let selected_exists =
                        entries.iter().any(|(source, _)| source == &trait_name.name);
                    if !selected_exists {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0026",
                            trait_name.span,
                            format!(
                                "trait `{}` does not contribute `{}`",
                                trait_name.name, method.name
                            ),
                        ));
                    }
                    let contributing = entries
                        .iter()
                        .map(|(source, _)| source.as_str())
                        .collect::<BTreeSet<_>>();
                    for excluded_trait in excluded {
                        if excluded_trait.name == trait_name.name
                            || !contributing.contains(excluded_trait.name.as_str())
                        {
                            self.diagnostics.push(Diagnostic::error(
                                "typing",
                                "T0026",
                                excluded_trait.span,
                                format!(
                                    "trait `{}` cannot be excluded from `{}::{}`",
                                    excluded_trait.name, trait_name.name, method.name
                                ),
                            ));
                        }
                    }
                    let excluded = excluded
                        .iter()
                        .map(|name| name.name.as_str())
                        .collect::<BTreeSet<_>>();
                    entries.retain(|(source, _)| {
                        source == &trait_name.name || !excluded.contains(source.as_str())
                    });
                }
            }
            let mut selected = BTreeMap::new();
            for (name, entries) in candidates {
                if entries.len() > 1 {
                    let mut diagnostic = Diagnostic::error(
                        "typing",
                        "T0027",
                        trait_use.span,
                        format!("trait method `{name}` has multiple contributors"),
                    );
                    for (_, method) in &entries {
                        diagnostic = diagnostic.with_label(method.span, "contribution is here");
                    }
                    self.diagnostics.push(diagnostic);
                }
                if let Some((_, method)) = entries.into_iter().next() {
                    selected.insert(name, method);
                }
            }
            for adaptation in &trait_use.adaptations {
                let TraitAdaptation::Alias {
                    trait_name,
                    method,
                    visibility,
                    final_method,
                    alias,
                    span,
                } = adaptation
                else {
                    continue;
                };
                let Some((_, source)) = original_candidates
                    .get(&method.name)
                    .and_then(|entries| {
                        entries
                            .iter()
                            .find(|(source, _)| source == &trait_name.name)
                    })
                    .cloned()
                else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0026",
                        *span,
                        format!(
                            "unknown trait method `{}::{}`",
                            trait_name.name, method.name
                        ),
                    ));
                    continue;
                };
                let target_name = alias
                    .as_ref()
                    .map_or_else(|| method.name.clone(), |alias| alias.name.clone());
                if alias.is_none() {
                    let Some(selected_method) = selected.get_mut(&target_name) else {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0026",
                            *span,
                            format!(
                                "trait method `{}::{}` is not selected for adaptation",
                                trait_name.name, method.name
                            ),
                        ));
                        continue;
                    };
                    if selected_method.origin_trait.as_deref() != Some(trait_name.name.as_str()) {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0026",
                            *span,
                            format!(
                                "trait method `{}::{}` is not the selected contribution",
                                trait_name.name, method.name
                            ),
                        ));
                        continue;
                    }
                    if let Some(visibility) = visibility {
                        selected_method.visibility = *visibility;
                    }
                    selected_method.final_method |= *final_method;
                    continue;
                }
                let mut adapted = source;
                if let Some(visibility) = visibility {
                    adapted.visibility = *visibility;
                }
                adapted.final_method |= *final_method;
                if let Some(previous) = selected.get(&target_name) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "typing",
                            "T0027",
                            *span,
                            format!("trait alias `{target_name}` collides with another method"),
                        )
                        .with_label(previous.span, "existing method is here"),
                    );
                } else {
                    selected.insert(target_name, adapted);
                }
            }
            for (name, mut method) in selected {
                method.declaring_class = consumer.id;
                method.slot = self.method_slots[&name];
                if consumer.kind == NominalKind::Class
                    && !method.abstract_method
                    && let Some(source) = method.source.clone()
                {
                    let id = self.allocate_function();
                    method.signature.id = id;
                    method.callee = Some(Callee::Function(id));
                    self.pending_methods.push(PendingMethod {
                        id,
                        owner: consumer.id,
                        declaration: MethodDecl {
                            function: FunctionDecl {
                                name: name.clone(),
                                ..source.function.clone()
                            },
                            ..source
                        },
                        signature: method.signature.clone(),
                        span: method.span,
                    });
                }
                if let Some(previous) = methods.get(&name) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "typing",
                            "T0027",
                            method.span,
                            format!("trait method `{name}` has multiple contributors"),
                        )
                        .with_label(previous.span, "first contribution is here"),
                    );
                } else {
                    methods.insert(name, method);
                }
            }
        }
        (properties, initializers, methods)
    }

    fn validate_override(
        &mut self,
        class_name: &str,
        method_name: &str,
        replacement: &MethodSignature,
        parent: &MethodSignature,
    ) {
        if parent.visibility == Visibility::Private {
            self.diagnostics.push(
                Diagnostic::error(
                    "typing",
                    "T0028",
                    replacement.span,
                    format!(
                        "`{class_name}::{method_name}` redeclares a parent-private method name"
                    ),
                )
                .with_label(parent.span, "private method is declared here"),
            );
        }
        if parent.final_method {
            self.diagnostics.push(
                Diagnostic::error(
                    "typing",
                    "T0029",
                    replacement.span,
                    format!("final method `{method_name}` cannot be replaced"),
                )
                .with_label(parent.span, "final method is declared here"),
            );
        }
        if !method_contract_equal(replacement, parent) {
            self.diagnostics.push(
                Diagnostic::error(
                    "typing",
                    "T0030",
                    replacement.span,
                    format!("override `{class_name}::{method_name}` must match exactly"),
                )
                .with_label(parent.span, "overridden signature is here"),
            );
        }
        if visibility_rank(replacement.visibility) < visibility_rank(parent.visibility) {
            self.diagnostics.push(
                Diagnostic::error(
                    "typing",
                    "T0031",
                    replacement.span,
                    format!("override `{class_name}::{method_name}` narrows visibility"),
                )
                .with_label(parent.span, "overridden visibility is declared here"),
            );
        }
    }
}

struct FunctionChecker<'signatures, 'diagnostics> {
    signatures: &'signatures BTreeMap<String, Signature>,
    classes: &'signatures BTreeMap<String, ClassSignature>,
    diagnostics: &'diagnostics mut Vec<Diagnostic>,
    id: FunctionId,
    name: String,
    return_type: Type,
    span: Span,
    locals: Vec<Local>,
    names: HashMap<String, LocalId>,
    parameters: Vec<LocalId>,
    loop_depth: usize,
    owner: Option<ClassId>,
    static_method: bool,
}

impl<'signatures, 'diagnostics> FunctionChecker<'signatures, 'diagnostics> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        signatures: &'signatures BTreeMap<String, Signature>,
        classes: &'signatures BTreeMap<String, ClassSignature>,
        diagnostics: &'diagnostics mut Vec<Diagnostic>,
        id: FunctionId,
        name: &str,
        return_type: Type,
        span: Span,
        owner: Option<ClassId>,
        static_method: bool,
    ) -> Self {
        Self {
            signatures,
            classes,
            diagnostics,
            id,
            name: name.to_owned(),
            return_type,
            span,
            locals: Vec::new(),
            names: HashMap::new(),
            parameters: Vec::new(),
            loop_depth: 0,
            owner,
            static_method,
        }
    }

    fn declare_receiver(&mut self, class: &ClassSignature) {
        let id = self.add_local(
            "this".to_owned(),
            Type::Object(class.name.clone()),
            class.span,
            true,
        );
        self.parameters.push(id);
    }

    fn declare_parameters(
        &mut self,
        declaration: &FunctionDecl,
        signatures: &[ParameterSignature],
    ) {
        for (parameter, signature) in declaration.parameters.iter().zip(signatures) {
            let ty = &signature.ty;
            if let Some(previous) = self.names.get(&parameter.name).copied() {
                let previous_span = self.locals[previous.0 as usize].span;
                self.diagnostics.push(
                    Diagnostic::error(
                        "name_resolution",
                        "N0002",
                        parameter.name_span,
                        format!("parameter `${}` is declared more than once", parameter.name),
                    )
                    .with_label(previous_span, "first parameter is here"),
                );
                continue;
            }
            let id = self.add_local(
                parameter.name.clone(),
                ty.clone(),
                parameter.name_span,
                true,
            );
            self.parameters.push(id);
            if let Some(default) = &parameter.default {
                if !is_default_constant(default) {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0310",
                        default.span,
                        "parameter defaults must be typed constant expressions",
                    ));
                } else if let Some(value) = self.lower_expression(default, Some(ty)) {
                    self.expect_type(ty, &value.ty, value.span);
                }
            }
        }
    }

    fn add_local(&mut self, name: String, ty: Type, span: Span, parameter: bool) -> LocalId {
        let id =
            LocalId(u32::try_from(self.locals.len()).expect("local count is limited to u32::MAX"));
        self.names.insert(name.clone(), id);
        self.locals.push(Local {
            id,
            name,
            ty,
            span,
            parameter,
        });
        id
    }

    fn lower_block(&mut self, statements: &Block) -> Vec<Statement> {
        statements
            .iter()
            .filter_map(|statement| self.lower_statement(statement))
            .collect()
    }

    fn lower_statement(&mut self, statement: &Stmt) -> Option<Statement> {
        let kind = match &statement.kind {
            StmtKind::Function(_)
            | StmtKind::Class(_)
            | StmtKind::Interface(_)
            | StmtKind::Trait(_) => {
                self.diagnostics.push(Diagnostic::error(
                    "name_resolution",
                    "N0003",
                    statement.span,
                    "nested declarations are not supported",
                ));
                return None;
            }
            StmtKind::Assign {
                name,
                annotation,
                value,
            } => {
                let annotated = annotation
                    .as_ref()
                    .and_then(|syntax| resolve_type(syntax, self.classes, self.diagnostics));
                if let Some(id) = self.names.get(name).copied() {
                    if annotation.is_some() {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0001",
                            annotation.as_ref().expect("annotation exists").span,
                            format!("`${name}` is already declared and cannot be redeclared"),
                        ));
                    }
                    let expected = self.locals[id.0 as usize].ty.clone();
                    let value = self.lower_expression(value, Some(&expected))?;
                    self.expect_type(&expected, &value.ty, value.span);
                    StatementKind::Assign {
                        local: id,
                        value,
                        declares: false,
                    }
                } else {
                    let value = self.lower_expression(value, annotated.as_ref())?;
                    let ty = annotated.unwrap_or_else(|| value.ty.clone());
                    self.expect_type(&ty, &value.ty, value.span);
                    let id = self.add_local(name.clone(), ty, statement.span, false);
                    StatementKind::Assign {
                        local: id,
                        value,
                        declares: true,
                    }
                }
            }
            StmtKind::Echo(expression) => {
                let value = self.lower_expression(expression, None)?;
                if !value.ty.is_output_scalar() {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0002",
                        value.span,
                        format!(
                            "`echo` expects `string`, `int`, `float`, or `bool`, got `{}`",
                            value.ty
                        ),
                    ));
                }
                StatementKind::Echo(value)
            }
            StmtKind::Return(expression) => {
                let return_type = self.return_type.clone();
                let value = expression
                    .as_ref()
                    .and_then(|expression| self.lower_expression(expression, Some(&return_type)));
                match (&return_type, &value) {
                    (Type::Void, Some(value)) => self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0003",
                        value.span,
                        "a `void` function cannot return a value",
                    )),
                    (expected, Some(value)) => self.expect_type(expected, &value.ty, value.span),
                    (Type::Void, None) => {}
                    (expected, None) => self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0004",
                        statement.span,
                        format!("function must return a value of type `{expected}`"),
                    )),
                }
                StatementKind::Return(value)
            }
            StmtKind::If {
                branches,
                otherwise,
            } => {
                let branches = branches
                    .iter()
                    .filter_map(|(condition, body)| {
                        let condition = self.lower_expression(condition, Some(&Type::Bool))?;
                        self.expect_type(&Type::Bool, &condition.ty, condition.span);
                        Some((condition, self.lower_block(body)))
                    })
                    .collect();
                let otherwise = otherwise
                    .as_ref()
                    .map_or_else(Vec::new, |body| self.lower_block(body));
                StatementKind::If {
                    branches,
                    otherwise,
                }
            }
            StmtKind::While { condition, body } => {
                let condition = self.lower_expression(condition, Some(&Type::Bool))?;
                self.expect_type(&Type::Bool, &condition.ty, condition.span);
                self.loop_depth += 1;
                let body = self.lower_block(body);
                self.loop_depth -= 1;
                StatementKind::While { condition, body }
            }
            StmtKind::For {
                initializers,
                conditions,
                updates,
                body,
            } => {
                let initializers = initializers
                    .iter()
                    .filter_map(|clause| self.lower_loop_clause(clause))
                    .collect::<Vec<_>>();
                let conditions = conditions
                    .iter()
                    .filter_map(|clause| self.lower_loop_clause(clause))
                    .collect::<Vec<_>>();
                if let Some(condition) = conditions.last() {
                    self.expect_type(&Type::Bool, loop_clause_type(condition), condition.span);
                }
                let updates = updates
                    .iter()
                    .filter_map(|clause| self.lower_loop_clause(clause))
                    .collect::<Vec<_>>();
                self.loop_depth += 1;
                let body = self.lower_block(body);
                self.loop_depth -= 1;
                StatementKind::For {
                    initializers,
                    conditions,
                    updates,
                    body,
                }
            }
            StmtKind::Foreach {
                source,
                key,
                value,
                body,
            } => {
                let source = self.lower_expression(source, None)?;
                let (key_type, value_type) = match &source.ty {
                    Type::Vector(value) => (Type::Int, value.as_ref().clone()),
                    Type::Map(key, value) => (key.as_ref().clone(), value.as_ref().clone()),
                    other => {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0610",
                            source.span,
                            format!(
                                "`foreach` requires `vector<T>` or `map<K, V>`, found `{other}`"
                            ),
                        ));
                        return None;
                    }
                };
                if key.as_ref().is_some_and(|key| key.name == value.name) {
                    self.diagnostics.push(Diagnostic::error(
                        "name_resolution",
                        "N0610",
                        value.span,
                        "foreach key and value variables must be different",
                    ));
                    return None;
                }
                let key_binding = key
                    .as_ref()
                    .map(|binding| self.bind_foreach_local(&binding.name, &key_type, binding.span));
                let value_binding = self.bind_foreach_local(&value.name, &value_type, value.span);
                self.loop_depth += 1;
                let lowered_body = self.lower_block(body);
                self.loop_depth -= 1;
                if let Some((_, previous, name)) = &key_binding {
                    self.restore_name(name, *previous);
                }
                self.restore_name(&value.name, value_binding.1);
                StatementKind::Foreach {
                    source,
                    key: key_binding.map(|(local, _, _)| local),
                    value: value_binding.0,
                    key_type,
                    value_type,
                    body: lowered_body,
                }
            }
            StmtKind::Break | StmtKind::Continue => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        if matches!(statement.kind, StmtKind::Break) {
                            "T0620"
                        } else {
                            "T0621"
                        },
                        statement.span,
                        if matches!(statement.kind, StmtKind::Break) {
                            "`break` is only valid inside a loop"
                        } else {
                            "`continue` is only valid inside a loop"
                        },
                    ));
                }
                if matches!(statement.kind, StmtKind::Break) {
                    StatementKind::Break
                } else {
                    StatementKind::Continue
                }
            }
            StmtKind::SetProperty {
                object,
                property,
                property_span,
                value,
            } => {
                let object = self.lower_expression(object, None)?;
                let (property_id, property_type) =
                    self.lookup_property(&object.ty, property, *property_span)?;
                let value = self.lower_expression(value, Some(&property_type))?;
                self.expect_type(&property_type, &value.ty, value.span);
                StatementKind::SetProperty {
                    object,
                    property: property_id,
                    value,
                }
            }
            StmtKind::SetIndex {
                root,
                root_span,
                indices,
                value,
            } => {
                let (root, collection_types, indices, value) =
                    self.lower_index_assignment(root, *root_span, indices, value)?;
                StatementKind::SetIndex {
                    root,
                    collection_types,
                    indices,
                    value,
                }
            }
            StmtKind::Throw(value) => {
                let value = self.lower_expression(value, None)?;
                if !type_is_subtype_of(&value.ty, "Throwable", self.classes) {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0008",
                        value.span,
                        format!(
                            "thrown expressions must be `Throwable`, found `{}`",
                            value.ty
                        ),
                    ));
                }
                StatementKind::Throw(value)
            }
            StmtKind::Try {
                body,
                catches,
                finally,
            } => {
                let body = self.lower_block(body);
                let mut previous_catches = Vec::<ClassId>::new();
                let catches = catches
                    .iter()
                    .filter_map(|clause| {
                        let Some(class) = self.classes.get(&clause.class_name).cloned() else {
                            self.diagnostics.push(Diagnostic::error(
                                "name_resolution",
                                "N0104",
                                clause.class_span,
                                format!("unknown class `{}`", clause.class_name),
                            ));
                            return None;
                        };
                        if !is_nominal_subtype(self.classes, &class.name, "Throwable") {
                            self.diagnostics.push(Diagnostic::error(
                                "typing",
                                "T0450",
                                clause.class_span,
                                format!("catch type `{}` is not throwable", class.name),
                            ));
                        }
                        if let Some(previous) = previous_catches.iter().find(|previous| {
                            let previous =
                                &self.classes.values().find(|class| class.id == **previous);
                            previous.is_some_and(|previous| {
                                is_nominal_subtype(self.classes, &class.name, &previous.name)
                            })
                        }) {
                            let previous_name = self
                                .classes
                                .values()
                                .find(|candidate| candidate.id == *previous)
                                .map_or("<unknown>", |candidate| candidate.name.as_str());
                            self.diagnostics.push(Diagnostic::error(
                                "typing",
                                "T0451",
                                clause.class_span,
                                format!(
                                    "catch `{}` is already handled by earlier `{previous_name}`",
                                    class.name
                                ),
                            ));
                        }
                        previous_catches.push(class.id);
                        let previous = self.names.get(&clause.variable).copied();
                        let local = self.add_local(
                            clause.variable.clone(),
                            Type::Object(class.name),
                            clause.variable_span,
                            false,
                        );
                        let body = self.lower_block(&clause.body);
                        if let Some(previous) = previous {
                            self.names.insert(clause.variable.clone(), previous);
                        } else {
                            self.names.remove(&clause.variable);
                        }
                        Some(Catch {
                            class: class.id,
                            local,
                            body,
                            span: clause.span,
                        })
                    })
                    .collect();
                let finally = finally.as_ref().map(|body| self.lower_block(body));
                StatementKind::Try {
                    body,
                    catches,
                    finally,
                }
            }
            StmtKind::Using {
                variable,
                variable_span,
                value,
                body,
            } => {
                let value = self.lower_expression(value, None)?;
                let Type::Object(class_name) = &value.ty else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0601",
                        value.span,
                        format!("`using` requires a closeable object, found `{}`", value.ty),
                    ));
                    return None;
                };
                if !is_nominal_subtype(self.classes, class_name, "Closeable") {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0601",
                        value.span,
                        format!("`using` requires a nominal `Closeable`, found `{class_name}`"),
                    ));
                }
                let class = self.classes.get(class_name)?;
                let close = if let Some(close) = class.methods.get("close").cloned() {
                    if close.static_method
                        || !close.signature.parameters.is_empty()
                        || close.signature.return_type != Type::Void
                    {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0603",
                            close.span,
                            "`using` cleanup must be an instance `close(): void` method",
                        ));
                    }
                    close.slot
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0602",
                        value.span,
                        format!("class `{class_name}` does not define `close()`"),
                    ));
                    return None;
                };
                let previous = self.names.get(variable).copied();
                let local =
                    self.add_local(variable.clone(), value.ty.clone(), *variable_span, false);
                let body = self.lower_block(body);
                if let Some(previous) = previous {
                    self.names.insert(variable.clone(), previous);
                } else {
                    self.names.remove(variable);
                }
                StatementKind::Using {
                    local,
                    value,
                    close,
                    body,
                }
            }
            StmtKind::Block(body) => StatementKind::Block(self.lower_block(body)),
            StmtKind::Expression(expression) => {
                StatementKind::Expression(self.lower_expression(expression, None)?)
            }
        };
        Some(Statement {
            kind,
            span: statement.span,
        })
    }

    fn lower_loop_clause(&mut self, clause: &ForClause) -> Option<LoopClause> {
        let kind = match &clause.kind {
            ForClauseKind::Assign {
                name,
                name_span,
                annotation,
                value,
            } => {
                let annotated = annotation
                    .as_ref()
                    .and_then(|syntax| resolve_type(syntax, self.classes, self.diagnostics));
                let local = if let Some(local) = self.names.get(name).copied() {
                    if annotation.is_some() {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0001",
                            annotation.as_ref().expect("annotation exists").span,
                            format!("`${name}` is already declared and cannot be redeclared"),
                        ));
                    }
                    local
                } else {
                    let inferred = annotated.clone().unwrap_or(Type::Mixed);
                    self.add_local(name.clone(), inferred, *name_span, false)
                };
                let expected =
                    annotated.unwrap_or_else(|| self.locals[local.0 as usize].ty.clone());
                let value = self.lower_expression(value, Some(&expected))?;
                if self.locals[local.0 as usize].ty == Type::Mixed {
                    self.locals[local.0 as usize].ty = value.ty.clone();
                } else {
                    self.expect_type(&expected, &value.ty, value.span);
                }
                LoopClauseKind::Assign { local, value }
            }
            ForClauseKind::SetProperty {
                object,
                property,
                property_span,
                value,
            } => {
                let object = self.lower_expression(object, None)?;
                let (property, property_type) =
                    self.lookup_property(&object.ty, property, *property_span)?;
                let value = self.lower_expression(value, Some(&property_type))?;
                self.expect_type(&property_type, &value.ty, value.span);
                LoopClauseKind::SetProperty {
                    object,
                    property,
                    value,
                }
            }
            ForClauseKind::SetIndex {
                root,
                root_span,
                indices,
                value,
            } => {
                let (root, collection_types, indices, value) =
                    self.lower_index_assignment(root, *root_span, indices, value)?;
                LoopClauseKind::SetIndex {
                    root,
                    collection_types,
                    indices,
                    value,
                }
            }
            ForClauseKind::Expression(expression) => {
                LoopClauseKind::Expression(self.lower_expression(expression, None)?)
            }
        };
        Some(LoopClause {
            kind,
            span: clause.span,
        })
    }

    fn bind_foreach_local(
        &mut self,
        name: &str,
        ty: &Type,
        span: Span,
    ) -> (LocalId, Option<LocalId>, String) {
        if let Some(local) = self.names.get(name).copied() {
            let existing = self.locals[local.0 as usize].ty.clone();
            self.expect_type(&existing, ty, span);
            return (local, Some(local), name.to_owned());
        }
        (
            self.add_local(name.to_owned(), ty.clone(), span, false),
            None,
            name.to_owned(),
        )
    }

    fn restore_name(&mut self, name: &str, previous: Option<LocalId>) {
        if let Some(previous) = previous {
            self.names.insert(name.to_owned(), previous);
        } else {
            self.names.remove(name);
        }
    }

    fn lower_index_assignment(
        &mut self,
        root_name: &str,
        root_span: Span,
        indices: &[Expr],
        value: &Expr,
    ) -> Option<(LocalId, Vec<Type>, Vec<TypedExpr>, TypedExpr)> {
        let Some(root) = self.names.get(root_name).copied() else {
            self.diagnostics.push(Diagnostic::error(
                "name_resolution",
                "N0101",
                root_span,
                format!("unknown variable `${root_name}`"),
            ));
            return None;
        };
        let mut current = self.locals[root.0 as usize].ty.clone();
        let mut collection_types = Vec::with_capacity(indices.len());
        let mut lowered_indices = Vec::with_capacity(indices.len());
        for index in indices {
            let (index_type, element_type) = match &current {
                Type::Vector(element) => (Type::Int, element.as_ref().clone()),
                Type::Map(key, value) => (key.as_ref().clone(), value.as_ref().clone()),
                other => {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0110",
                        index.span,
                        format!(
                            "collection assignment requires `vector<T>` or `map<K, V>`, found `{other}`"
                        ),
                    ));
                    return None;
                }
            };
            collection_types.push(current.clone());
            let index = self.lower_expression(index, Some(&index_type))?;
            self.expect_type(&index_type, &index.ty, index.span);
            lowered_indices.push(index);
            current = element_type;
        }
        let value = self.lower_expression(value, Some(&current))?;
        self.expect_type(&current, &value.ty, value.span);
        Some((root, collection_types, lowered_indices, value))
    }

    fn lower_expression(
        &mut self,
        expression: &Expr,
        expected: Option<&Type>,
    ) -> Option<TypedExpr> {
        let (kind, ty) = match &expression.kind {
            ExprKind::Integer(value) => (TypedExprKind::Integer(*value), Type::Int),
            ExprKind::Float(value) => (TypedExprKind::Float(*value), Type::Float),
            ExprKind::Bool(value) => (TypedExprKind::Bool(*value), Type::Bool),
            ExprKind::Null => (TypedExprKind::Null, Type::Null),
            ExprKind::String(value) => (TypedExprKind::String(value.clone()), Type::String),
            ExprKind::Variable(name) => {
                let Some(id) = self.names.get(name).copied() else {
                    self.diagnostics.push(Diagnostic::error(
                        "name_resolution",
                        "N0101",
                        expression.span,
                        format!("unknown variable `${name}`"),
                    ));
                    return None;
                };
                (
                    TypedExprKind::Local(id),
                    self.locals[id.0 as usize].ty.clone(),
                )
            }
            ExprKind::Name(name) => {
                self.diagnostics.push(Diagnostic::error(
                    "name_resolution",
                    "N0102",
                    expression.span,
                    format!("name `{name}` is only valid as a callable in this language subset"),
                ));
                return None;
            }
            ExprKind::Vector(values) => {
                let expected_element = match expected {
                    Some(Type::Vector(element)) => Some(element.as_ref()),
                    _ => None,
                };
                let values = values
                    .iter()
                    .filter_map(|value| self.lower_expression(value, expected_element))
                    .collect::<Vec<_>>();
                let element = if let Some(expected) = expected_element {
                    for value in &values {
                        self.expect_type(expected, &value.ty, value.span);
                    }
                    expected.clone()
                } else if let Some(first) = values.first() {
                    let element = first.ty.clone();
                    for value in &values[1..] {
                        self.expect_type(&element, &value.ty, value.span);
                    }
                    element
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0101",
                        expression.span,
                        "cannot infer the element type of an empty vector",
                    ));
                    Type::Mixed
                };
                (
                    TypedExprKind::Vector(values),
                    Type::Vector(Box::new(element)),
                )
            }
            ExprKind::Map(entries) => {
                let (expected_key, expected_value) = match expected {
                    Some(Type::Map(key, value)) => (Some(key.as_ref()), Some(value.as_ref())),
                    _ => (None, None),
                };
                let entries = entries
                    .iter()
                    .filter_map(|entry| {
                        let key = self.lower_expression(&entry.key, expected_key)?;
                        let value = self.lower_expression(&entry.value, expected_value)?;
                        Some((key, value))
                    })
                    .collect::<Vec<_>>();
                let key_type = expected_key
                    .cloned()
                    .or_else(|| entries.first().map(|(key, _)| key.ty.clone()));
                let value_type = expected_value
                    .cloned()
                    .or_else(|| entries.first().map(|(_, value)| value.ty.clone()));
                let (key_type, value_type) =
                    if let (Some(key), Some(value)) = (key_type, value_type) {
                        (key, value)
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0102",
                            expression.span,
                            "cannot infer key and value types of an empty map",
                        ));
                        (Type::Mixed, Type::Mixed)
                    };
                for (key, value) in &entries {
                    self.expect_type(&key_type, &key.ty, key.span);
                    self.expect_type(&value_type, &value.ty, value.span);
                }
                (
                    TypedExprKind::Map(entries),
                    Type::Map(Box::new(key_type), Box::new(value_type)),
                )
            }
            ExprKind::Unary { op, operand } => {
                let expected_operand = match op {
                    UnaryOp::Negate => None,
                    UnaryOp::Not => Some(&Type::Bool),
                };
                let operand = self.lower_expression(operand, expected_operand)?;
                let ty = match op {
                    UnaryOp::Negate if matches!(operand.ty, Type::Int | Type::Float) => {
                        operand.ty.clone()
                    }
                    UnaryOp::Negate => {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0201",
                            operand.span,
                            format!("unary `-` requires a number, found `{}`", operand.ty),
                        ));
                        Type::Mixed
                    }
                    UnaryOp::Not => {
                        self.expect_type(&Type::Bool, &operand.ty, operand.span);
                        Type::Bool
                    }
                };
                (
                    TypedExprKind::Unary {
                        op: *op,
                        operand: Box::new(operand),
                    },
                    ty,
                )
            }
            ExprKind::Binary { op, left, right } => {
                let left = self.lower_expression(left, None)?;
                let right = self.lower_expression(right, Some(&left.ty))?;
                let ty = self.binary_type(*op, &left, &right);
                (
                    TypedExprKind::Binary {
                        op: *op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty,
                )
            }
            ExprKind::Call { callee, arguments } => {
                let ExprKind::Name(name) = &callee.kind else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0301",
                        callee.span,
                        "dynamic callable invocation is not supported",
                    ));
                    return None;
                };
                return self.lower_call(name, arguments, expression.span);
            }
            ExprKind::Index { collection, index } => {
                let collection = self.lower_expression(collection, None)?;
                let (index_type, result_type) = match &collection.ty {
                    Type::Vector(element) => (Type::Int, element.as_ref().clone()),
                    Type::Map(key, value) => (key.as_ref().clone(), value.as_ref().clone()),
                    other => {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0401",
                            collection.span,
                            format!("values of type `{other}` cannot be indexed"),
                        ));
                        (Type::Mixed, Type::Mixed)
                    }
                };
                let index = self.lower_expression(index, Some(&index_type))?;
                self.expect_type(&index_type, &index.ty, index.span);
                (
                    TypedExprKind::Index {
                        collection: Box::new(collection),
                        index: Box::new(index),
                    },
                    result_type,
                )
            }
            ExprKind::New {
                class_name,
                class_span,
                arguments,
            } => {
                let Some(class) = self.classes.get(class_name).cloned() else {
                    self.diagnostics.push(Diagnostic::error(
                        "name_resolution",
                        "N0104",
                        *class_span,
                        format!("unknown class `{class_name}`"),
                    ));
                    return None;
                };
                if class.kind != NominalKind::Class {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0409",
                        *class_span,
                        format!(
                            "{} `{class_name}` cannot be instantiated",
                            nominal_kind_name(class.kind)
                        ),
                    ));
                    return None;
                }
                if class.abstract_class {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0410",
                        *class_span,
                        format!("abstract class `{class_name}` cannot be instantiated"),
                    ));
                }
                if class.native && !is_nominal_subtype(self.classes, class_name, "Throwable") {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0409",
                        *class_span,
                        format!("native class `{class_name}` cannot be constructed directly"),
                    ));
                    return None;
                }
                let constructor = class.constructor;
                let initializers = class
                    .properties
                    .iter()
                    .zip(&class.property_initializers)
                    .filter_map(|(property, initializer)| {
                        let initializer = initializer.as_ref()?;
                        let value = self.lower_expression(initializer, Some(&property.ty))?;
                        self.expect_type(&property.ty, &value.ty, value.span);
                        Some((property.id, value))
                    })
                    .collect();
                let arguments = if constructor.is_some() {
                    let method = class
                        .methods
                        .get("__construct")
                        .expect("constructor id has a method signature");
                    self.bind_arguments(
                        "__construct",
                        arguments,
                        &method.signature.parameters,
                        expression.span,
                    )
                } else {
                    self.bind_arguments("__construct", arguments, &[], expression.span)
                };
                (
                    TypedExprKind::New {
                        class: class.id,
                        constructor,
                        initializers,
                        arguments,
                    },
                    Type::Object(class.name),
                )
            }
            ExprKind::Property {
                object,
                name,
                name_span,
            } => {
                let object = self.lower_expression(object, None)?;
                let (property, ty) = self.lookup_property(&object.ty, name, *name_span)?;
                (
                    TypedExprKind::Property {
                        object: Box::new(object),
                        property,
                    },
                    ty,
                )
            }
            ExprKind::MethodCall {
                object,
                name,
                name_span,
                arguments,
            } => {
                let object = self.lower_expression(object, None)?;
                let Type::Object(class_name) = object.ty.clone() else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0402",
                        object.span,
                        format!("method calls require an object, found `{}`", object.ty),
                    ));
                    return None;
                };
                let Some(class) = self.classes.get(&class_name) else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0403",
                        object.span,
                        format!("class metadata for `{class_name}` is unavailable"),
                    ));
                    return None;
                };
                let Some(method) = class.methods.get(name).cloned() else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0404",
                        *name_span,
                        format!("method `{name}` is not defined for `{class_name}`"),
                    ));
                    return None;
                };
                self.check_member_access(
                    method.declaring_class,
                    method.visibility,
                    *name_span,
                    "method",
                    name,
                );
                if method.static_method {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0405",
                        *name_span,
                        format!("static method `{class_name}::{name}` requires `::`"),
                    ));
                }
                let lowered_arguments = self.bind_arguments(
                    name,
                    arguments,
                    &method.signature.parameters,
                    expression.span,
                );
                let call = if method.visibility == Visibility::Private || method.final_method {
                    let Some(callee) = method.callee else {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0411",
                            *name_span,
                            format!("abstract method `{class_name}::{name}` cannot be called"),
                        ));
                        return None;
                    };
                    TypedExprKind::DirectMethod {
                        callee,
                        receiver: Some(Box::new(object)),
                        arguments: lowered_arguments,
                        called_class: CalledClass::Receiver,
                    }
                } else {
                    TypedExprKind::VirtualMethod {
                        receiver: Box::new(object),
                        slot: method.slot,
                        arguments: lowered_arguments,
                    }
                };
                (call, method.signature.return_type)
            }
            ExprKind::StaticCall {
                target,
                class_span,
                name,
                name_span,
                arguments,
            } => {
                if let ScopeTarget::Named(class_name) = target
                    && self
                        .classes
                        .get(class_name)
                        .is_some_and(|class| class.native)
                    && !self.classes[class_name].methods.contains_key(name)
                {
                    return self.lower_native_static(class_name, name, arguments, expression.span);
                }
                let (class_name, called_class, late_static) = match target {
                    ScopeTarget::Named(class_name) => {
                        let Some(class) = self.classes.get(class_name) else {
                            self.diagnostics.push(Diagnostic::error(
                                "name_resolution",
                                "N0104",
                                *class_span,
                                format!("unknown class `{class_name}`"),
                            ));
                            return None;
                        };
                        (class.name.clone(), CalledClass::Explicit(class.id), false)
                    }
                    ScopeTarget::SelfType => {
                        let Some(owner) = self.owner_class() else {
                            self.diagnostics.push(Diagnostic::error(
                                "typing",
                                "T0412",
                                *class_span,
                                "`self::` is available only in a class method",
                            ));
                            return None;
                        };
                        (owner.name.clone(), CalledClass::Forwarded, false)
                    }
                    ScopeTarget::Parent => {
                        let Some(owner) = self.owner_class() else {
                            self.diagnostics.push(Diagnostic::error(
                                "typing",
                                "T0412",
                                *class_span,
                                "`parent::` is available only in a class method",
                            ));
                            return None;
                        };
                        let Some(parent) = owner.parent.as_ref() else {
                            self.diagnostics.push(Diagnostic::error(
                                "typing",
                                "T0413",
                                *class_span,
                                format!("class `{}` has no parent", owner.name),
                            ));
                            return None;
                        };
                        (parent.clone(), CalledClass::Forwarded, false)
                    }
                    ScopeTarget::Static => {
                        let Some(owner) = self.owner_class() else {
                            self.diagnostics.push(Diagnostic::error(
                                "typing",
                                "T0412",
                                *class_span,
                                "`static::` is available only in a class method",
                            ));
                            return None;
                        };
                        (owner.name.clone(), CalledClass::Forwarded, true)
                    }
                };
                let Some(class) = self.classes.get(&class_name) else {
                    self.diagnostics.push(Diagnostic::error(
                        "name_resolution",
                        "N0104",
                        *class_span,
                        format!("unknown class `{class_name}`"),
                    ));
                    return None;
                };
                let Some(method) = class.methods.get(name).cloned() else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0404",
                        *name_span,
                        format!("method `{name}` is not defined for `{class_name}`"),
                    ));
                    return None;
                };
                self.check_member_access(
                    method.declaring_class,
                    method.visibility,
                    *name_span,
                    "method",
                    name,
                );
                let arguments = self.bind_arguments(
                    name,
                    arguments,
                    &method.signature.parameters,
                    expression.span,
                );
                let receiver = if method.static_method {
                    None
                } else {
                    if self.static_method {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0406",
                            *name_span,
                            format!(
                                "static context cannot call instance method `{class_name}::{name}`"
                            ),
                        ));
                        return None;
                    }
                    let Some(receiver) = self.receiver_expression(*class_span) else {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0406",
                            *name_span,
                            format!(
                                "instance method `{class_name}::{name}` requires an object context"
                            ),
                        ));
                        return None;
                    };
                    if matches!(target, ScopeTarget::Named(_))
                        && self.owner_class().is_some_and(|owner| {
                            !is_nominal_subtype(self.classes, &owner.name, &class_name)
                        })
                    {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0406",
                            *name_span,
                            format!(
                                "the current receiver is not compatible with `{class_name}::{name}`"
                            ),
                        ));
                        return None;
                    }
                    Some(Box::new(receiver))
                };
                let call = if late_static {
                    TypedExprKind::LateStaticMethod {
                        receiver,
                        slot: method.slot,
                        arguments,
                    }
                } else {
                    let Some(callee) = method.callee else {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0411",
                            *name_span,
                            format!("abstract method `{class_name}::{name}` cannot be called"),
                        ));
                        return None;
                    };
                    TypedExprKind::DirectMethod {
                        callee,
                        receiver,
                        arguments,
                        called_class,
                    }
                };
                (call, method.signature.return_type)
            }
            ExprKind::ClassConstant {
                class_name,
                class_span,
                name,
                name_span,
            } => {
                let value = match (class_name.as_str(), name.as_str()) {
                    ("OpenMode", "Read") | ("SeekFrom", "Start") => 0,
                    ("OpenMode", "Write") | ("SeekFrom", "Current") => 1,
                    ("OpenMode", "ReadWrite") | ("SeekFrom", "End") => 2,
                    _ => {
                        self.diagnostics.push(Diagnostic::error(
                            "name_resolution",
                            "N0105",
                            *name_span,
                            format!("unknown class constant `{class_name}::{name}`"),
                        ));
                        return None;
                    }
                };
                if !matches!(class_name.as_str(), "OpenMode" | "SeekFrom") {
                    self.diagnostics.push(Diagnostic::error(
                        "name_resolution",
                        "N0104",
                        *class_span,
                        format!("unknown native enum `{class_name}`"),
                    ));
                }
                (TypedExprKind::Integer(value), Type::Int)
            }
            ExprKind::InstanceOf {
                value,
                class_name,
                class_span,
            } => {
                let value = self.lower_expression(value, None)?;
                let Some(class) = self.classes.get(class_name) else {
                    self.diagnostics.push(Diagnostic::error(
                        "name_resolution",
                        "N0104",
                        *class_span,
                        format!("unknown class `{class_name}`"),
                    ));
                    return None;
                };
                (
                    TypedExprKind::InstanceOf {
                        value: Box::new(value),
                        class: class.id,
                    },
                    Type::Bool,
                )
            }
            ExprKind::Match { subject, arms } => {
                return self.lower_match_expression(subject, arms, expected, expression.span);
            }
        };
        Some(TypedExpr {
            kind,
            ty,
            span: expression.span,
        })
    }

    fn lower_match_expression(
        &mut self,
        subject: &Expr,
        arms: &[MatchArm],
        expected: Option<&Type>,
        span: Span,
    ) -> Option<TypedExpr> {
        let subject = self.lower_expression(subject, None)?;
        let mut lowered_arms = Vec::with_capacity(arms.len());
        let mut result_types = Vec::new();
        let mut default_span = None;
        let mut literal_conditions = BTreeMap::<String, Span>::new();
        for arm in arms {
            if arm.default {
                if let Some(previous) = default_span {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "typing",
                            "T0702",
                            arm.span,
                            "a match expression may contain only one `default` arm",
                        )
                        .with_label(previous, "first default arm is here"),
                    );
                } else {
                    default_span = Some(arm.span);
                }
            }
            let mut conditions = Vec::with_capacity(arm.conditions.len());
            for condition in &arm.conditions {
                let condition = self.lower_expression(condition, None)?;
                if !types_overlap(&subject.ty, &condition.ty, self.classes) {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0701",
                        condition.span,
                        format!(
                            "match condition type `{}` cannot match subject type `{}`",
                            condition.ty, subject.ty
                        ),
                    ));
                }
                if let Some(key) = literal_key(&condition.kind)
                    && let Some(previous) = literal_conditions.insert(key, condition.span)
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "typing",
                            "T0703",
                            condition.span,
                            "duplicate literal match condition",
                        )
                        .with_label(previous, "first matching literal is here"),
                    );
                }
                conditions.push(condition);
            }
            let value = self.lower_expression(&arm.value, expected)?;
            if let Some(expected) = expected {
                self.expect_type(expected, &value.ty, value.span);
            }
            if value.ty != Type::Never {
                result_types.push(value.ty.clone());
            }
            lowered_arms.push(TypedMatchArm {
                conditions,
                value,
                default: arm.default,
                span: arm.span,
            });
        }
        let ty = if result_types.is_empty() {
            expected.cloned().unwrap_or(Type::Never)
        } else {
            normalize_union(result_types)
        };
        if let Some(expected) = expected {
            self.expect_type(expected, &ty, span);
        }
        Some(TypedExpr {
            kind: TypedExprKind::Match {
                subject: Box::new(subject),
                arms: lowered_arms,
            },
            ty,
            span,
        })
    }

    fn lower_call(&mut self, name: &str, arguments: &[Argument], span: Span) -> Option<TypedExpr> {
        if name == "var_dump" {
            let mut explicit = Vec::new();
            for (index, argument) in arguments.iter().enumerate() {
                if argument.name.is_some() {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0318",
                        argument.name_span.unwrap_or(argument.span),
                        "named arguments cannot target a variadic parameter",
                    ));
                }
                if let Some(value) = self.lower_expression(&argument.value, None) {
                    explicit.push(BoundArgument {
                        target: ArgumentTarget::Parameter(index),
                        value,
                    });
                }
            }
            return Some(TypedExpr {
                kind: TypedExprKind::Call {
                    callee: Callee::Builtin(Builtin::VarDump),
                    arguments: BoundArguments {
                        explicit,
                        defaults: Vec::new(),
                        parameter_count: arguments.len(),
                        variadic_parameter: None,
                        variadic_type: None,
                    },
                },
                ty: Type::Void,
                span,
            });
        }
        if name == "count" {
            let parameters = vec![native_parameter("value", Type::Mixed, None, span)];
            let arguments = self.bind_arguments(name, arguments, &parameters, span);
            for argument in arguments.explicit.iter().chain(&arguments.defaults) {
                if !matches!(
                    argument.value.ty,
                    Type::String | Type::Vector(_) | Type::Map(_, _)
                ) {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0302",
                        argument.value.span,
                        format!(
                            "`count` cannot count a value of type `{}`",
                            argument.value.ty
                        ),
                    ));
                }
            }
            return Some(TypedExpr {
                kind: TypedExprKind::Call {
                    callee: Callee::Builtin(Builtin::Count),
                    arguments,
                },
                ty: Type::Int,
                span,
            });
        }
        let Some(signature) = self.signatures.get(name).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                "name_resolution",
                "N0103",
                span,
                format!("unknown function `{name}`"),
            ));
            return None;
        };
        let arguments = self.bind_arguments(name, arguments, &signature.parameters, span);
        Some(TypedExpr {
            kind: TypedExprKind::Call {
                callee: Callee::Function(signature.id),
                arguments,
            },
            ty: signature.return_type.clone(),
            span,
        })
    }

    fn lower_native_static(
        &mut self,
        class_name: &str,
        name: &str,
        arguments: &[Argument],
        span: Span,
    ) -> Option<TypedExpr> {
        let (callee, parameters, return_type) = match (class_name, name) {
            ("MemoryStream", "open") => (
                Builtin::MemoryStreamOpen,
                vec![native_parameter(
                    "initial",
                    Type::String,
                    Some(ExprKind::String(Vec::new())),
                    span,
                )],
                Type::Object("MemoryStream".to_owned()),
            ),
            ("TempStream", "open") => (
                Builtin::TempStreamOpen,
                vec![native_parameter(
                    "maxMemoryBytes",
                    Type::Int,
                    Some(ExprKind::Integer(2_097_152)),
                    span,
                )],
                Type::Object("TempStream".to_owned()),
            ),
            ("Streams", "open") => {
                let return_type = match arguments.first().map(|argument| &argument.value.kind) {
                    Some(ExprKind::String(uri)) if uri == b"thp:/input" => {
                        Type::Object("ReadableStream".to_owned())
                    }
                    Some(ExprKind::String(uri)) if uri.starts_with(b"php://temp") => {
                        Type::Object("TempStream".to_owned())
                    }
                    _ => Type::Object("MemoryStream".to_owned()),
                };
                (
                    Builtin::StreamsOpen,
                    vec![
                        native_parameter("uri", Type::String, None, span),
                        native_parameter("mode", Type::Int, None, span),
                    ],
                    return_type,
                )
            }
            ("Files", "openRead") => (
                Builtin::FilesOpenRead,
                vec![native_parameter("path", Type::String, None, span)],
                Type::Object("ReadableFileStream".to_owned()),
            ),
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "typing",
                    "T0404",
                    span,
                    format!("static method `{class_name}::{name}` is not defined"),
                ));
                return None;
            }
        };
        let arguments = self.bind_arguments(
            &format!("{class_name}::{name}"),
            arguments,
            &parameters,
            span,
        );
        Some(TypedExpr {
            kind: TypedExprKind::Call {
                callee: Callee::Builtin(callee),
                arguments,
            },
            ty: return_type,
            span,
        })
    }

    fn bind_arguments(
        &mut self,
        name: &str,
        arguments: &[Argument],
        parameters: &[ParameterSignature],
        span: Span,
    ) -> BoundArguments {
        let variadic_parameter = parameters.iter().position(|parameter| parameter.variadic);
        let mut occupied = vec![false; parameters.len()];
        let mut bound_spans = vec![None; parameters.len()];
        let mut explicit = Vec::new();
        let mut next_positional = 0;
        let mut saw_named = false;
        for argument in arguments {
            let target = if let Some(argument_name) = &argument.name {
                saw_named = true;
                match parameters
                    .iter()
                    .position(|parameter| parameter.name == *argument_name)
                {
                    Some(index) if parameters[index].variadic => {
                        self.diagnostics.push(Diagnostic::error(
                            "typing",
                            "T0318",
                            argument.name_span.unwrap_or(argument.span),
                            "named arguments cannot target a variadic parameter",
                        ));
                        continue;
                    }
                    Some(index) => ArgumentTarget::Parameter(index),
                    None => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "typing",
                                "T0314",
                                argument.name_span.unwrap_or(argument.span),
                                format!("unknown named argument `{argument_name}` for `{name}`"),
                            )
                            .with_note(format!(
                                "known parameters: {}",
                                parameters
                                    .iter()
                                    .map(|parameter| parameter.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )),
                        );
                        continue;
                    }
                }
            } else {
                if saw_named {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0313",
                        argument.span,
                        "positional arguments cannot follow named arguments",
                    ));
                }
                while next_positional < parameters.len()
                    && occupied[next_positional]
                    && !parameters[next_positional].variadic
                {
                    next_positional += 1;
                }
                if next_positional < parameters.len() && !parameters[next_positional].variadic {
                    let target = ArgumentTarget::Parameter(next_positional);
                    next_positional += 1;
                    target
                } else if variadic_parameter.is_some() {
                    ArgumentTarget::Variadic
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0303",
                        argument.span,
                        format!("function `{name}` received too many arguments"),
                    ));
                    continue;
                }
            };
            let (expected, duplicate) = match target {
                ArgumentTarget::Parameter(index) => {
                    let duplicate = occupied[index];
                    occupied[index] = true;
                    if !duplicate {
                        bound_spans[index] = Some(argument.span);
                    }
                    (&parameters[index].ty, duplicate)
                }
                ArgumentTarget::Variadic => {
                    let index = variadic_parameter.expect("variadic target has a parameter");
                    let Type::Vector(element) = &parameters[index].ty else {
                        unreachable!("variadic parameters use vector locals")
                    };
                    (element.as_ref(), false)
                }
            };
            if duplicate {
                let index = match target {
                    ArgumentTarget::Parameter(index) => index,
                    ArgumentTarget::Variadic => unreachable!(),
                };
                self.diagnostics.push(
                    Diagnostic::error(
                        "typing",
                        "T0315",
                        argument.span,
                        format!(
                            "parameter `{}` is bound more than once",
                            parameters[index].name
                        ),
                    )
                    .with_label(
                        bound_spans[index].unwrap_or(parameters[index].span),
                        "first binding is here",
                    )
                    .with_label(parameters[index].span, "parameter is declared here"),
                );
                continue;
            }
            if let Some(value) = self.lower_expression(&argument.value, Some(expected)) {
                self.expect_type(expected, &value.ty, value.span);
                explicit.push(BoundArgument { target, value });
            }
        }
        let mut defaults = Vec::new();
        for (index, parameter) in parameters.iter().enumerate() {
            if parameter.variadic || occupied[index] {
                continue;
            }
            if let Some(default) = &parameter.default {
                if let Some(value) = self.lower_expression(default, Some(&parameter.ty)) {
                    self.expect_type(&parameter.ty, &value.ty, value.span);
                    defaults.push(BoundArgument {
                        target: ArgumentTarget::Parameter(index),
                        value,
                    });
                }
            } else {
                self.diagnostics.push(
                    Diagnostic::error(
                        "typing",
                        "T0316",
                        span,
                        format!(
                            "function `{name}` is missing required argument `{}`",
                            parameter.name
                        ),
                    )
                    .with_label(parameter.span, "required parameter is declared here"),
                );
            }
        }
        BoundArguments {
            explicit,
            defaults,
            parameter_count: parameters.len(),
            variadic_parameter,
            variadic_type: variadic_parameter.and_then(|index| {
                let Type::Vector(element) = &parameters[index].ty else {
                    return None;
                };
                Some(element.as_ref().clone())
            }),
        }
    }

    fn lookup_property(
        &mut self,
        object_type: &Type,
        name: &str,
        span: Span,
    ) -> Option<(PropertyId, Type)> {
        let Type::Object(class_name) = object_type else {
            self.diagnostics.push(Diagnostic::error(
                "typing",
                "T0407",
                span,
                format!("property access requires an object, found `{object_type}`"),
            ));
            return None;
        };
        let Some(class) = self.classes.get(class_name) else {
            self.diagnostics.push(Diagnostic::error(
                "typing",
                "T0403",
                span,
                format!("class metadata for `{class_name}` is unavailable"),
            ));
            return None;
        };
        let Some(property) = class
            .properties
            .iter()
            .find(|property| property.name == name)
        else {
            self.diagnostics.push(Diagnostic::error(
                "typing",
                "T0408",
                span,
                format!("property `{name}` is not defined for `{class_name}`"),
            ));
            return None;
        };
        let property = property.clone();
        self.check_member_access(
            property.declaring_class,
            property.visibility,
            span,
            "property",
            name,
        );
        Some((property.id, property.ty.clone()))
    }

    fn owner_class(&self) -> Option<&ClassSignature> {
        let owner = self.owner?;
        self.classes.values().find(|class| class.id == owner)
    }

    fn receiver_expression(&mut self, span: Span) -> Option<TypedExpr> {
        let local = self.names.get("this").copied()?;
        let ty = self.locals[local.0 as usize].ty.clone();
        Some(TypedExpr {
            kind: TypedExprKind::Local(local),
            ty,
            span,
        })
    }

    fn check_member_access(
        &mut self,
        declaring_class: ClassId,
        visibility: Visibility,
        span: Span,
        member_kind: &str,
        name: &str,
    ) {
        let allowed = match visibility {
            Visibility::Public => true,
            Visibility::Private => self.owner == Some(declaring_class),
            Visibility::Protected => self.owner.is_some_and(|owner| {
                owner == declaring_class
                    || is_class_id_subtype(self.classes, owner, declaring_class)
            }),
        };
        if !allowed {
            let declaring = self
                .classes
                .values()
                .find(|class| class.id == declaring_class)
                .map_or("<unknown>", |class| class.name.as_str());
            self.diagnostics.push(Diagnostic::error(
                "typing",
                "T0414",
                span,
                format!(
                    "{visibility:?} {member_kind} `{declaring}::{name}` is not accessible here"
                ),
            ));
        }
    }

    fn binary_type(&mut self, op: BinaryOp, left: &TypedExpr, right: &TypedExpr) -> Type {
        match op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Remainder => {
                if left.ty == right.ty && matches!(left.ty, Type::Int | Type::Float) {
                    left.ty.clone()
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0501",
                        left.span.join(right.span),
                        format!(
                            "numeric operands must have the same numeric type, found `{}` and `{}`",
                            left.ty, right.ty
                        ),
                    ));
                    Type::Mixed
                }
            }
            BinaryOp::Concatenate => {
                if left.ty.is_output_scalar() && right.ty.is_output_scalar() {
                    Type::String
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0502",
                        left.span.join(right.span),
                        "concatenation accepts only `string`, `int`, `float`, or `bool` values",
                    ));
                    Type::String
                }
            }
            BinaryOp::Equal | BinaryOp::StrictEqual | BinaryOp::NotEqual => {
                if left.ty != right.ty {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0503",
                        right.span,
                        format!(
                            "comparison requires matching operand types, found `{}` and `{}`",
                            left.ty, right.ty
                        ),
                    ));
                }
                Type::Bool
            }
            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                if left.ty != right.ty || !matches!(left.ty, Type::Int | Type::Float | Type::String)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0504",
                        left.span.join(right.span),
                        "ordered comparison requires matching int, float, or string operands",
                    ));
                }
                Type::Bool
            }
            BinaryOp::And | BinaryOp::Or => {
                self.expect_type(&Type::Bool, &left.ty, left.span);
                self.expect_type(&Type::Bool, &right.ty, right.span);
                Type::Bool
            }
            BinaryOp::Coalesce => {
                let Some(non_null) = left.ty.without_null() else {
                    return right.ty.clone();
                };
                if !left.ty.accepts(&Type::Null) {
                    self.diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0505",
                        left.span,
                        format!("left operand of `??` cannot be null (`{}`)", left.ty),
                    ));
                }
                self.expect_type(&non_null, &right.ty, right.span);
                non_null
            }
        }
    }

    fn expect_type(&mut self, expected: &Type, actual: &Type, span: Span) {
        if !type_accepts(expected, actual, self.classes) {
            self.diagnostics.push(Diagnostic::error(
                "typing",
                "T0005",
                span,
                format!("expected `{expected}`, found `{actual}`"),
            ));
        }
    }

    fn finish(self, body: Vec<Statement>) -> Function {
        if self.return_type != Type::Void && !block_guarantees_exit(&body) {
            self.diagnostics.push(Diagnostic::error(
                "typing",
                "T0009",
                self.span,
                format!(
                    "function `{}` may complete without returning `{}`",
                    self.name, self.return_type
                ),
            ));
        }
        Function {
            id: self.id,
            name: self.name,
            parameters: self.parameters,
            locals: self.locals,
            return_type: self.return_type,
            owner: self.owner,
            static_method: self.static_method,
            body,
            span: self.span,
        }
    }
}

fn block_guarantees_exit(statements: &[Statement]) -> bool {
    statements
        .last()
        .is_some_and(|statement| match &statement.kind {
            StatementKind::Return(_) | StatementKind::Throw(_) => true,
            StatementKind::Block(body) | StatementKind::Using { body, .. } => {
                block_guarantees_exit(body)
            }
            StatementKind::If {
                branches,
                otherwise,
            } => {
                !otherwise.is_empty()
                    && branches.iter().all(|(_, body)| block_guarantees_exit(body))
                    && block_guarantees_exit(otherwise)
            }
            StatementKind::Try {
                body,
                catches,
                finally,
            } => {
                finally
                    .as_ref()
                    .is_some_and(|body| block_guarantees_exit(body))
                    || (block_guarantees_exit(body)
                        && catches
                            .iter()
                            .all(|catch| block_guarantees_exit(&catch.body)))
            }
            StatementKind::Assign { .. }
            | StatementKind::Echo(_)
            | StatementKind::While { .. }
            | StatementKind::For { .. }
            | StatementKind::Foreach { .. }
            | StatementKind::Break
            | StatementKind::Continue
            | StatementKind::SetProperty { .. }
            | StatementKind::SetIndex { .. }
            | StatementKind::Expression(_) => false,
        })
}

fn resolve_parameters(
    declaration: &FunctionDecl,
    classes: &BTreeMap<String, ClassSignature>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ParameterSignature> {
    let mut optional_seen = false;
    let mut variadic_seen = false;
    declaration
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            let element_type =
                resolve_type(&parameter.ty, classes, diagnostics).unwrap_or(Type::Mixed);
            if parameter.variadic {
                if variadic_seen {
                    diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0306",
                        parameter.span,
                        "a function may declare only one variadic parameter",
                    ));
                }
                if index + 1 != declaration.parameters.len() {
                    diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0307",
                        parameter.span,
                        "a variadic parameter must be the final parameter",
                    ));
                }
                if parameter.default.is_some() {
                    diagnostics.push(Diagnostic::error(
                        "typing",
                        "T0308",
                        parameter.span,
                        "a variadic parameter cannot have a default value",
                    ));
                }
                variadic_seen = true;
            } else if parameter.default.is_some() {
                optional_seen = true;
            } else if optional_seen {
                diagnostics.push(Diagnostic::error(
                    "typing",
                    "T0309",
                    parameter.span,
                    "a required parameter cannot follow a parameter with a default",
                ));
            }
            ParameterSignature {
                name: parameter.name.clone(),
                ty: if parameter.variadic {
                    Type::Vector(Box::new(element_type))
                } else {
                    element_type
                },
                default: parameter.default.clone(),
                variadic: parameter.variadic,
                span: parameter.span,
            }
        })
        .collect()
}

fn native_parameter(
    name: &str,
    ty: Type,
    default: Option<ExprKind>,
    span: Span,
) -> ParameterSignature {
    ParameterSignature {
        name: name.to_owned(),
        ty,
        default: default.map(|kind| Expr { kind, span }),
        variadic: false,
        span,
    }
}

fn is_default_constant(expression: &Expr) -> bool {
    match &expression.kind {
        ExprKind::Integer(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Null
        | ExprKind::String(_) => true,
        ExprKind::Vector(values) => values.iter().all(is_default_constant),
        ExprKind::Map(entries) => entries
            .iter()
            .all(|entry| is_default_constant(&entry.key) && is_default_constant(&entry.value)),
        ExprKind::Unary { operand, .. } => is_default_constant(operand),
        _ => false,
    }
}

fn types_overlap(left: &Type, right: &Type, classes: &BTreeMap<String, ClassSignature>) -> bool {
    left == &Type::Mixed
        || right == &Type::Mixed
        || type_accepts(left, right, classes)
        || type_accepts(right, left, classes)
}

fn type_accepts(
    expected: &Type,
    actual: &Type,
    classes: &BTreeMap<String, ClassSignature>,
) -> bool {
    expected == &Type::Mixed
        || actual == &Type::Never
        || expected == actual
        || matches!(
            actual,
            Type::Union(members)
                if members
                    .iter()
                    .all(|member| type_accepts(expected, member, classes))
        )
        || matches!(
            expected,
            Type::Union(members)
                if members
                    .iter()
                    .any(|member| type_accepts(member, actual, classes))
        )
        || matches!(
            (expected, actual),
            (Type::Object(expected), Type::Object(actual))
                if is_nominal_subtype(classes, actual, expected)
        )
}

fn type_is_subtype_of(
    actual: &Type,
    expected: &str,
    classes: &BTreeMap<String, ClassSignature>,
) -> bool {
    match actual {
        Type::Object(actual) => is_nominal_subtype(classes, actual, expected),
        Type::Union(members) => members
            .iter()
            .all(|member| type_is_subtype_of(member, expected, classes)),
        _ => false,
    }
}

fn is_nominal_subtype(
    classes: &BTreeMap<String, ClassSignature>,
    actual: &str,
    expected: &str,
) -> bool {
    if actual == expected {
        return true;
    }
    let Some(actual) = classes.get(actual) else {
        return false;
    };
    if actual
        .interfaces
        .iter()
        .any(|interface| interface == expected)
    {
        return true;
    }
    actual
        .parent
        .as_deref()
        .is_some_and(|parent| is_nominal_subtype(classes, parent, expected))
}

fn is_class_id_subtype(
    classes: &BTreeMap<String, ClassSignature>,
    actual: ClassId,
    expected: ClassId,
) -> bool {
    let actual = classes.values().find(|class| class.id == actual);
    let expected = classes.values().find(|class| class.id == expected);
    match (actual, expected) {
        (Some(actual), Some(expected)) => is_nominal_subtype(classes, &actual.name, &expected.name),
        _ => false,
    }
}

fn literal_key(kind: &TypedExprKind) -> Option<String> {
    match kind {
        TypedExprKind::Integer(value) => Some(format!("int:{value}")),
        TypedExprKind::Float(value) => Some(format!("float:{:016x}", value.to_bits())),
        TypedExprKind::Bool(value) => Some(format!("bool:{value}")),
        TypedExprKind::Null => Some("null".to_owned()),
        TypedExprKind::String(value) => Some(format!("string:{value:?}")),
        TypedExprKind::Unary { op, operand } => {
            literal_key(&operand.kind).map(|value| format!("unary:{op:?}:{value}"))
        }
        _ => None,
    }
}

fn loop_clause_type(clause: &LoopClause) -> &Type {
    match &clause.kind {
        LoopClauseKind::Assign { value, .. }
        | LoopClauseKind::SetProperty { value, .. }
        | LoopClauseKind::SetIndex { value, .. }
        | LoopClauseKind::Expression(value) => &value.ty,
    }
}

fn resolve_type(
    syntax: &TypeSyntax,
    classes: &BTreeMap<String, ClassSignature>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Type> {
    match &syntax.kind {
        TypeSyntaxKind::Nullable(inner) => {
            let inner = resolve_type(inner, classes, diagnostics)?;
            Some(normalize_union(vec![inner, Type::Null]))
        }
        TypeSyntaxKind::Union(members) => Some(normalize_union(
            members
                .iter()
                .filter_map(|member| resolve_type(member, classes, diagnostics))
                .collect(),
        )),
        TypeSyntaxKind::Named { name, arguments } => {
            let arity = arguments.len();
            let primitive = match name.as_str() {
                "int" => Some(Type::Int),
                "float" => Some(Type::Float),
                "bool" => Some(Type::Bool),
                "string" => Some(Type::String),
                "null" => Some(Type::Null),
                "void" => Some(Type::Void),
                "never" => Some(Type::Never),
                "mixed" => Some(Type::Mixed),
                _ => None,
            };
            if let Some(primitive) = primitive {
                if arity != 0 {
                    diagnostics.push(Diagnostic::error(
                        "typing",
                        "T1001",
                        syntax.span,
                        format!("type `{name}` does not accept generic arguments"),
                    ));
                }
                return Some(primitive);
            }
            match name.as_str() {
                "vector" if arity == 1 => Some(Type::Vector(Box::new(resolve_type(
                    &arguments[0],
                    classes,
                    diagnostics,
                )?))),
                "map" if arity == 2 => Some(Type::Map(
                    Box::new(resolve_type(&arguments[0], classes, diagnostics)?),
                    Box::new(resolve_type(&arguments[1], classes, diagnostics)?),
                )),
                "vector" | "map" => {
                    let expected = if name == "vector" { 1 } else { 2 };
                    diagnostics.push(Diagnostic::error(
                        "typing",
                        "T1002",
                        syntax.span,
                        format!(
                            "type `{name}` expects {expected} generic arguments, found {arity}"
                        ),
                    ));
                    None
                }
                _ if arity == 0
                    && classes
                        .get(name)
                        .is_some_and(|class| class.kind != NominalKind::Trait) =>
                {
                    Some(Type::Object(name.clone()))
                }
                _ if arity == 0
                    && classes
                        .get(name)
                        .is_some_and(|class| class.kind == NominalKind::Trait) =>
                {
                    diagnostics.push(Diagnostic::error(
                        "typing",
                        "T1004",
                        syntax.span,
                        format!("trait `{name}` cannot be used as a type"),
                    ));
                    None
                }
                _ => {
                    diagnostics.push(Diagnostic::error(
                        "typing",
                        "T1003",
                        syntax.span,
                        format!("unknown type `{name}`"),
                    ));
                    None
                }
            }
        }
    }
}

type NativeNominalSpec<'name> = (
    &'name str,
    NominalKind,
    bool,
    bool,
    Option<&'name str>,
    &'name [&'name str],
);

fn native_nominals() -> BTreeMap<String, ClassSignature> {
    let specs: &[NativeNominalSpec<'_>] = &[
        ("Closeable", NominalKind::Interface, true, false, None, &[]),
        (
            "ReadableStream",
            NominalKind::Interface,
            true,
            false,
            Some("Closeable"),
            &[],
        ),
        (
            "WritableStream",
            NominalKind::Interface,
            true,
            false,
            Some("Closeable"),
            &[],
        ),
        (
            "SeekableStream",
            NominalKind::Interface,
            true,
            false,
            Some("Closeable"),
            &[],
        ),
        ("Throwable", NominalKind::Interface, true, false, None, &[]),
        (
            "MemoryStream",
            NominalKind::Class,
            false,
            true,
            None,
            &["ReadableStream", "WritableStream", "SeekableStream"],
        ),
        (
            "TempStream",
            NominalKind::Class,
            false,
            true,
            None,
            &["ReadableStream", "WritableStream", "SeekableStream"],
        ),
        ("Streams", NominalKind::Class, false, true, None, &[]),
        ("Files", NominalKind::Class, false, true, None, &[]),
        (
            "ReadableFileStream",
            NominalKind::Class,
            false,
            true,
            None,
            &["ReadableStream"],
        ),
        ("OpenMode", NominalKind::Class, false, true, None, &[]),
        ("SeekFrom", NominalKind::Class, false, true, None, &[]),
        (
            "Exception",
            NominalKind::Class,
            false,
            false,
            None,
            &["Throwable"],
        ),
        (
            "ValueError",
            NominalKind::Class,
            false,
            true,
            Some("Exception"),
            &[],
        ),
        (
            "IoException",
            NominalKind::Class,
            false,
            false,
            Some("Exception"),
            &[],
        ),
        (
            "OpenStreamException",
            NominalKind::Class,
            false,
            false,
            Some("IoException"),
            &[],
        ),
        (
            "ClosedStreamException",
            NominalKind::Class,
            false,
            true,
            Some("IoException"),
            &[],
        ),
        (
            "UnsupportedStreamOperationException",
            NominalKind::Class,
            false,
            true,
            Some("IoException"),
            &[],
        ),
        (
            "InvalidStreamUriException",
            NominalKind::Class,
            false,
            true,
            Some("OpenStreamException"),
            &[],
        ),
        (
            "Error",
            NominalKind::Class,
            false,
            false,
            None,
            &["Throwable"],
        ),
        (
            "UnhandledMatchError",
            NominalKind::Class,
            false,
            true,
            Some("Error"),
            &[],
        ),
    ];
    let mut classes = specs
        .iter()
        .enumerate()
        .map(
            |(index, (name, kind, abstract_class, final_class, parent, interfaces))| {
                (
                    (*name).to_owned(),
                    ClassSignature {
                        id: ClassId(
                            u32::try_from(index)
                                .expect("native nominal count is limited to u32::MAX"),
                        ),
                        name: (*name).to_owned(),
                        kind: *kind,
                        abstract_class: *abstract_class,
                        final_class: *final_class,
                        parent: parent.map(ToOwned::to_owned),
                        interfaces: interfaces
                            .iter()
                            .map(|interface| (*interface).to_owned())
                            .collect(),
                        trait_uses: Vec::new(),
                        declared_properties: Vec::new(),
                        declared_property_initializers: Vec::new(),
                        declared_methods: BTreeMap::new(),
                        properties: Vec::new(),
                        property_initializers: Vec::new(),
                        methods: BTreeMap::new(),
                        constructor: None,
                        native: true,
                        linked: false,
                        span: Span::empty(0),
                    },
                )
            },
        )
        .collect::<BTreeMap<_, _>>();

    let nullable_throwable =
        normalize_union(vec![Type::Object("Throwable".to_owned()), Type::Null]);
    let throwable_vector = Type::Vector(Box::new(Type::Object("Throwable".to_owned())));
    add_native_method(
        &mut classes,
        "Throwable",
        "getMessage",
        None,
        false,
        vec![],
        Type::String,
        true,
    );
    add_native_method(
        &mut classes,
        "Throwable",
        "getCode",
        None,
        false,
        vec![],
        Type::Int,
        true,
    );
    add_native_method(
        &mut classes,
        "Throwable",
        "getPrevious",
        None,
        false,
        vec![],
        nullable_throwable.clone(),
        true,
    );
    add_native_method(
        &mut classes,
        "Throwable",
        "getSuppressed",
        None,
        false,
        vec![],
        throwable_vector.clone(),
        true,
    );
    add_native_method(
        &mut classes,
        "Closeable",
        "close",
        None,
        false,
        vec![],
        Type::Void,
        true,
    );
    add_native_method(
        &mut classes,
        "Closeable",
        "isClosed",
        None,
        false,
        vec![],
        Type::Bool,
        true,
    );
    add_native_method(
        &mut classes,
        "ReadableStream",
        "tell",
        None,
        false,
        vec![],
        Type::Int,
        true,
    );
    add_native_method(
        &mut classes,
        "ReadableStream",
        "read",
        None,
        false,
        vec![native_parameter("length", Type::Int, None, Span::empty(0))],
        Type::String,
        true,
    );
    add_native_method(
        &mut classes,
        "ReadableStream",
        "readAll",
        None,
        false,
        vec![native_parameter(
            "limit",
            normalize_union(vec![Type::Int, Type::Null]),
            Some(ExprKind::Null),
            Span::empty(0),
        )],
        Type::String,
        true,
    );
    add_native_method(
        &mut classes,
        "ReadableStream",
        "eof",
        None,
        false,
        vec![],
        Type::Bool,
        true,
    );
    add_native_method(
        &mut classes,
        "WritableStream",
        "writeAll",
        None,
        false,
        vec![native_parameter(
            "bytes",
            Type::String,
            None,
            Span::empty(0),
        )],
        Type::Void,
        true,
    );
    add_native_method(
        &mut classes,
        "SeekableStream",
        "seek",
        None,
        false,
        vec![native_parameter(
            "position",
            Type::Int,
            None,
            Span::empty(0),
        )],
        Type::Void,
        true,
    );

    for class in ["Exception", "Error"] {
        add_native_method(
            &mut classes,
            class,
            "__construct",
            Some(Builtin::ExceptionConstruct),
            false,
            vec![
                native_parameter(
                    "message",
                    Type::String,
                    Some(ExprKind::String(Vec::new())),
                    Span::empty(0),
                ),
                native_parameter(
                    "code",
                    Type::Int,
                    Some(ExprKind::Integer(0)),
                    Span::empty(0),
                ),
                native_parameter(
                    "previous",
                    nullable_throwable.clone(),
                    Some(ExprKind::Null),
                    Span::empty(0),
                ),
            ],
            Type::Void,
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "getMessage",
            Some(Builtin::ExceptionGetMessage),
            false,
            vec![],
            Type::String,
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "getCode",
            Some(Builtin::ExceptionGetCode),
            false,
            vec![],
            Type::Int,
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "getPrevious",
            Some(Builtin::ExceptionGetPrevious),
            false,
            vec![],
            nullable_throwable.clone(),
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "getSuppressed",
            Some(Builtin::ExceptionGetSuppressed),
            false,
            vec![],
            throwable_vector.clone(),
            false,
        );
    }
    add_native_method(
        &mut classes,
        "OpenStreamException",
        "getTarget",
        Some(Builtin::ExceptionGetTarget),
        false,
        vec![],
        Type::String,
        false,
    );
    add_native_method(
        &mut classes,
        "OpenStreamException",
        "getSystemCode",
        Some(Builtin::ExceptionGetSystemCode),
        false,
        vec![],
        Type::Int,
        false,
    );

    for class in ["MemoryStream", "TempStream", "ReadableFileStream"] {
        add_native_method(
            &mut classes,
            class,
            "close",
            Some(Builtin::StreamClose),
            false,
            vec![],
            Type::Void,
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "isClosed",
            Some(Builtin::StreamIsClosed),
            false,
            vec![],
            Type::Bool,
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "tell",
            Some(Builtin::StreamTell),
            false,
            vec![],
            Type::Int,
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "read",
            Some(Builtin::StreamRead),
            false,
            vec![native_parameter("length", Type::Int, None, Span::empty(0))],
            Type::String,
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "readAll",
            Some(Builtin::StreamReadAll),
            false,
            vec![native_parameter(
                "limit",
                normalize_union(vec![Type::Int, Type::Null]),
                Some(ExprKind::Null),
                Span::empty(0),
            )],
            Type::String,
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "eof",
            Some(Builtin::StreamEof),
            false,
            vec![],
            Type::Bool,
            false,
        );
    }
    for class in ["MemoryStream", "TempStream"] {
        add_native_method(
            &mut classes,
            class,
            "writeAll",
            Some(Builtin::StreamWriteAll),
            false,
            vec![native_parameter(
                "bytes",
                Type::String,
                None,
                Span::empty(0),
            )],
            Type::Void,
            false,
        );
        add_native_method(
            &mut classes,
            class,
            "seek",
            Some(Builtin::StreamSeek),
            false,
            vec![native_parameter(
                "position",
                Type::Int,
                None,
                Span::empty(0),
            )],
            Type::Void,
            false,
        );
    }
    classes
}

#[allow(clippy::too_many_arguments)]
fn add_native_method(
    classes: &mut BTreeMap<String, ClassSignature>,
    class_name: &str,
    name: &str,
    builtin: Option<Builtin>,
    static_method: bool,
    parameters: Vec<ParameterSignature>,
    return_type: Type,
    abstract_method: bool,
) {
    let class = classes
        .get_mut(class_name)
        .expect("native nominal metadata names an existing type");
    let span = Span::empty(0);
    class.declared_methods.insert(
        name.to_owned(),
        MethodSignature {
            signature: Signature {
                id: FunctionId(u32::MAX),
                parameters,
                return_type,
                span,
            },
            callee: builtin.map(Callee::Builtin),
            slot: MethodSlot(u32::MAX),
            declaring_class: class.id,
            visibility: Visibility::Public,
            static_method,
            abstract_method,
            final_method: false,
            source: None,
            origin_trait: None,
            span,
        },
    );
}

const fn nominal_kind_name(kind: NominalKind) -> &'static str {
    match kind {
        NominalKind::Class => "class",
        NominalKind::Interface => "interface",
        NominalKind::Trait => "trait",
    }
}

fn method_contract_equal(left: &MethodSignature, right: &MethodSignature) -> bool {
    left.static_method == right.static_method
        && left.signature.return_type == right.signature.return_type
        && left.signature.parameters.len() == right.signature.parameters.len()
        && left
            .signature
            .parameters
            .iter()
            .zip(&right.signature.parameters)
            .all(|(left, right)| {
                left.name == right.name
                    && left.ty == right.ty
                    && left.variadic == right.variadic
                    && optional_constant_equal(left.default.as_ref(), right.default.as_ref())
            })
}

fn property_contract_equal(
    left: &Property,
    left_initializer: Option<&Expr>,
    right: &Property,
    right_initializer: Option<&Expr>,
) -> bool {
    left.name == right.name
        && left.ty == right.ty
        && left.visibility == right.visibility
        && optional_constant_equal(left_initializer, right_initializer)
}

fn optional_constant_equal(left: Option<&Expr>, right: Option<&Expr>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => constant_expression_equal(left, right),
        (None, Some(_)) | (Some(_), None) => false,
    }
}

fn constant_expression_equal(left: &Expr, right: &Expr) -> bool {
    match (&left.kind, &right.kind) {
        (ExprKind::Integer(left), ExprKind::Integer(right)) => left == right,
        (ExprKind::Float(left), ExprKind::Float(right)) => left.to_bits() == right.to_bits(),
        (ExprKind::Bool(left), ExprKind::Bool(right)) => left == right,
        (ExprKind::Null, ExprKind::Null) => true,
        (ExprKind::String(left), ExprKind::String(right)) => left == right,
        (ExprKind::Vector(left), ExprKind::Vector(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| constant_expression_equal(left, right))
        }
        (ExprKind::Map(left), ExprKind::Map(right)) => {
            left.len() == right.len()
                && left.iter().zip(right).all(|(left, right)| {
                    constant_expression_equal(&left.key, &right.key)
                        && constant_expression_equal(&left.value, &right.value)
                })
        }
        (
            ExprKind::Unary {
                op: left_op,
                operand: left,
            },
            ExprKind::Unary {
                op: right_op,
                operand: right,
            },
        ) => left_op == right_op && constant_expression_equal(left, right),
        _ => false,
    }
}

const fn visibility_rank(visibility: Visibility) -> u8 {
    match visibility {
        Visibility::Private => 0,
        Visibility::Protected => 1,
        Visibility::Public => 2,
    }
}

fn normalize_union(members: Vec<Type>) -> Type {
    let mut flat = Vec::new();
    for member in members {
        match member {
            Type::Union(nested) => flat.extend(nested),
            member => flat.push(member),
        }
    }
    flat.sort_by_key(ToString::to_string);
    flat.dedup();
    match flat.as_slice() {
        [single] => single.clone(),
        _ => Type::Union(flat),
    }
}

fn count_expressions(statements: &[Statement]) -> usize {
    statements
        .iter()
        .map(|statement| match &statement.kind {
            StatementKind::Assign { value, .. }
            | StatementKind::Expression(value)
            | StatementKind::Echo(value)
            | StatementKind::Throw(value) => count_expression(value),
            StatementKind::Return(value) => value.as_ref().map_or(0, count_expression),
            StatementKind::If {
                branches,
                otherwise,
            } => {
                branches
                    .iter()
                    .map(|(condition, body)| count_expression(condition) + count_expressions(body))
                    .sum::<usize>()
                    + count_expressions(otherwise)
            }
            StatementKind::While { condition, body } => {
                count_expression(condition) + count_expressions(body)
            }
            StatementKind::For {
                initializers,
                conditions,
                updates,
                body,
            } => {
                initializers
                    .iter()
                    .chain(conditions)
                    .chain(updates)
                    .map(count_loop_clause)
                    .sum::<usize>()
                    + count_expressions(body)
            }
            StatementKind::Foreach { source, body, .. } => {
                count_expression(source) + count_expressions(body)
            }
            StatementKind::Break | StatementKind::Continue => 0,
            StatementKind::SetProperty { object, value, .. } => {
                count_expression(object) + count_expression(value)
            }
            StatementKind::SetIndex { indices, value, .. } => {
                indices.iter().map(count_expression).sum::<usize>() + count_expression(value)
            }
            StatementKind::Try {
                body,
                catches,
                finally,
            } => {
                count_expressions(body)
                    + catches
                        .iter()
                        .map(|clause| count_expressions(&clause.body))
                        .sum::<usize>()
                    + finally.as_ref().map_or(0, |body| count_expressions(body))
            }
            StatementKind::Using { value, body, .. } => {
                count_expression(value) + count_expressions(body)
            }
            StatementKind::Block(body) => count_expressions(body),
        })
        .sum()
}

fn count_expression(expression: &TypedExpr) -> usize {
    1 + match &expression.kind {
        TypedExprKind::Unary { operand, .. } => count_expression(operand),
        TypedExprKind::Binary { left, right, .. } => {
            count_expression(left) + count_expression(right)
        }
        TypedExprKind::Call { arguments, .. } => count_bound_arguments(arguments),
        TypedExprKind::DirectMethod {
            receiver,
            arguments,
            ..
        }
        | TypedExprKind::LateStaticMethod {
            receiver,
            arguments,
            ..
        } => {
            receiver
                .as_ref()
                .map_or(0, |receiver| count_expression(receiver))
                + count_bound_arguments(arguments)
        }
        TypedExprKind::VirtualMethod {
            receiver,
            arguments,
            ..
        } => count_expression(receiver) + count_bound_arguments(arguments),
        TypedExprKind::Vector(arguments) => arguments.iter().map(count_expression).sum(),
        TypedExprKind::Map(entries) => entries
            .iter()
            .map(|(key, value)| count_expression(key) + count_expression(value))
            .sum(),
        TypedExprKind::Index { collection, index } => {
            count_expression(collection) + count_expression(index)
        }
        TypedExprKind::New {
            initializers,
            arguments,
            ..
        } => {
            initializers
                .iter()
                .map(|(_, value)| count_expression(value))
                .sum::<usize>()
                + count_bound_arguments(arguments)
        }
        TypedExprKind::Property { object, .. } => count_expression(object),
        TypedExprKind::InstanceOf { value, .. } => count_expression(value),
        TypedExprKind::Match { subject, arms } => {
            count_expression(subject)
                + arms
                    .iter()
                    .map(|arm| {
                        arm.conditions.iter().map(count_expression).sum::<usize>()
                            + count_expression(&arm.value)
                    })
                    .sum::<usize>()
        }
        TypedExprKind::Integer(_)
        | TypedExprKind::Float(_)
        | TypedExprKind::Bool(_)
        | TypedExprKind::Null
        | TypedExprKind::String(_)
        | TypedExprKind::Local(_) => 0,
    }
}

fn count_bound_arguments(arguments: &BoundArguments) -> usize {
    arguments
        .explicit
        .iter()
        .chain(&arguments.defaults)
        .map(|argument| count_expression(&argument.value))
        .sum()
}

fn count_loop_clause(clause: &LoopClause) -> usize {
    match &clause.kind {
        LoopClauseKind::Assign { value, .. } | LoopClauseKind::Expression(value) => {
            count_expression(value)
        }
        LoopClauseKind::SetProperty { object, value, .. } => {
            count_expression(object) + count_expression(value)
        }
        LoopClauseKind::SetIndex { indices, value, .. } => {
            indices.iter().map(count_expression).sum::<usize>() + count_expression(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use thp_diagnostics::SourceFile;
    use thp_syntax::parse;

    use super::{Type, lower};

    fn typecheck(source: &str) -> super::LowerOutput {
        let source = SourceFile::new("test.thp", source);
        let parsed = parse(&source);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        lower(&parsed.program)
    }

    fn diagnostic_codes(source: &str) -> Vec<&'static str> {
        typecheck(source)
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn propagates_function_and_vector_types() {
        let output = typecheck(
            r#"<?thp
function first(vector<int> $values): int {
    return $values[0];
}
$values: vector<int> = [1, 2];
$answer = first($values);
echo $answer . "";
"#,
        );
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let main = output.module.function(output.module.entry);
        assert_eq!(main.locals[1].ty, Type::Int);
    }

    #[test]
    fn rejects_wrong_assignment_type() {
        let output = typecheck("<?thp\n$value: int = \"wrong\";");
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "T0005")
        );
    }

    #[test]
    fn empty_collection_uses_expected_generic_type() {
        let output = typecheck("<?thp\n$values: vector<string> = [];");
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }

    #[test]
    fn echo_accepts_output_scalars_and_rejects_null() {
        let output = typecheck(
            "<?thp\nfunction answer(): int { return 42; }\necho answer();\necho 1.5;\necho true;\necho \"ok\";\necho null;",
        );
        let errors = output
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "T0002")
            .collect::<Vec<_>>();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("got `null`"));
    }

    #[test]
    fn concatenation_rejects_null() {
        let output = typecheck("<?thp\n$value = \"missing: \" . null;");
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "T0502")
        );
    }

    #[test]
    fn infers_match_unions_and_binds_argument_forms() {
        let output = typecheck(
            r#"<?thp
function select(int $base = 1, int ...$rest): int {
    return $base + count($rest);
}
$choice = match (1) { 1 => 42, default => "other" };
echo $choice . "";
echo select(base: 2) . select(2, 3, 4);
"#,
        );
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let main = output.module.function(output.module.entry);
        assert_eq!(
            main.locals[0].ty,
            Type::Union(vec![Type::Int, Type::String])
        );
    }

    #[test]
    fn reports_unknown_and_duplicate_named_arguments() {
        let output = typecheck(
            r"<?thp
function pair(int $left, int $right): int { return $left + $right; }
echo pair(left: 1, left: 2, other: 3);
",
        );
        let codes = output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"T0314"));
        assert!(codes.contains(&"T0315"));
        assert!(codes.contains(&"T0316"));
    }

    #[test]
    fn links_transitive_nominal_types_and_late_static_requirements() {
        let output = typecheck(
            r#"<?thp
interface Root {
    public function value(): string;
}
interface ChildContract extends Root {}
abstract class Base implements ChildContract {
    abstract public function value(): string;
    abstract public static function kind(): string;
    public function compatible(vector<int> $values = [1]): void {}
    public static function forwarded(): string {
        return static::kind();
    }
}
class Child extends Base {
    public function value(): string { return "child"; }
    public static function kind(): string { return "kind"; }
    public function compatible(vector<int> $values = [1]): void {}
}
function consume(Root $value): string {
    return $value->value();
}
$root: Root = new Child();
echo consume($root) . Child::forwarded();
"#,
        );
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let child = output
            .module
            .classes
            .iter()
            .find(|class| class.name == "Child")
            .unwrap();
        assert!(child.interfaces.iter().any(|interface| interface == "Root"));
    }

    #[test]
    fn diagnoses_nominal_kind_cycles_and_final_class_extension() {
        let codes = diagnostic_codes(
            r"<?thp
interface Contract {}
class Wrong extends Contract {}
class Left extends Right {}
class Right extends Left {}
final class Closed {}
class Reopened extends Closed {}
",
        );
        assert!(codes.contains(&"T0012"));
        assert!(codes.contains(&"T0011"));
        assert!(codes.contains(&"T0010"));
    }

    #[test]
    fn diagnoses_conflicting_requirements_incomplete_classes_and_overrides() {
        let codes = diagnostic_codes(
            r#"<?thp
interface First {
    public function render(string $value): string;
}
interface Second {
    public function render(int $value): string;
}
class Missing implements First, Second {}
class ParentType {
    public final function locked(string $value): string { return $value; }
    protected function exact(string $value = "x"): string { return $value; }
    public function __construct(string $name) {}
}
class ChildType extends ParentType {
    public function locked(string $value): string { return $value; }
    private function exact(string $renamed): string { return $renamed; }
    public function __construct(int $name) {}
}
"#,
        );
        assert!(codes.contains(&"T0022"));
        assert!(codes.contains(&"T0024"));
        assert!(codes.contains(&"T0029"));
        assert!(codes.contains(&"T0030"));
        assert!(codes.contains(&"T0031"));
    }

    #[test]
    fn diagnoses_visibility_property_and_trait_conflicts() {
        let codes = diagnostic_codes(
            r#"<?thp
trait FirstTrait {
    public function render(): string { return "first"; }
}
trait SecondTrait {
    public function render(): string { return "second"; }
}
class ParentType {
    private int $secret = 1;
    private function hidden(): int { return $this->secret; }
}
class ChildType extends ParentType {
    use FirstTrait, SecondTrait;
    public int $secret = 2;
    public function hidden(): int { return 2; }
}
$child = new ChildType();
echo $child->hidden();
$parent = new ParentType();
echo $parent->hidden();
"#,
        );
        assert!(codes.contains(&"T0027"));
        assert!(codes.contains(&"T0021"));
        assert!(codes.contains(&"T0028"));
        assert!(codes.contains(&"T0414"));
    }

    #[test]
    fn enforces_throwable_catch_and_closeable_nominality() {
        let codes = diagnostic_codes(
            r"<?thp
interface UserThrowable extends Throwable {}
class Plain {
    public function close(): void {}
}
throw new Plain();
try {
    throw new Exception();
} catch (Throwable $error) {
} catch (Exception $specific) {
}
using ($plain = new Plain()) {}
",
        );
        assert!(codes.contains(&"T0014"));
        assert!(codes.contains(&"T0008"));
        assert!(codes.contains(&"T0451"));
        assert!(codes.contains(&"T0601"));
    }
}
