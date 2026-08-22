use thp_diagnostics::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub namespace: Option<NamespaceDecl>,
    pub imports: Vec<UseDecl>,
    pub statements: Vec<Stmt>,
    pub span: Span,
}

/// A PHP-shaped name whose segments are retained separately for static
/// resolution. The leading separator is semantic and is not stored as an
/// empty segment.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    pub absolute: bool,
    pub segments: Vec<String>,
    pub span: Span,
}

impl QualifiedName {
    pub fn as_string(&self) -> String {
        let joined = self.segments.join("\\");
        if self.absolute {
            format!("\\{joined}")
        } else {
            joined
        }
    }

    pub fn last(&self) -> Option<&str> {
        self.segments.last().map(String::as_str)
    }
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.as_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceDecl {
    pub name: QualifiedName,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UseKind {
    Type,
    Function,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UseDecl {
    pub kind: UseKind,
    pub target: QualifiedName,
    pub alias: String,
    pub alias_span: Span,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    Function(FunctionDecl),
    Class(ClassDecl),
    Interface(InterfaceDecl),
    Trait(TraitDecl),
    Assign {
        name: String,
        annotation: Option<TypeSyntax>,
        value: Expr,
    },
    Echo(Expr),
    Return(Option<Expr>),
    If {
        branches: Vec<(Expr, Block)>,
        otherwise: Option<Block>,
    },
    While {
        condition: Expr,
        body: Block,
    },
    For {
        initializers: Vec<ForClause>,
        conditions: Vec<ForClause>,
        updates: Vec<ForClause>,
        body: Block,
    },
    Foreach {
        source: Expr,
        key: Option<LoopBinding>,
        value: LoopBinding,
        body: Block,
    },
    Break,
    Continue,
    SetProperty {
        object: Expr,
        property: String,
        property_span: Span,
        value: Expr,
    },
    SetIndex {
        root: String,
        root_span: Span,
        indices: Vec<Expr>,
        value: Expr,
    },
    Throw(Expr),
    Try {
        body: Block,
        catches: Vec<CatchClause>,
        finally: Option<Block>,
    },
    Using {
        variable: String,
        variable_span: Span,
        value: Expr,
        body: Block,
    },
    Block(Block),
    Expression(Expr),
}

pub type Block = Vec<Stmt>;

#[derive(Clone, Debug, PartialEq)]
pub struct LoopBinding {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForClause {
    pub kind: ForClauseKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ForClauseKind {
    Assign {
        name: String,
        name_span: Span,
        annotation: Option<TypeSyntax>,
        value: Expr,
    },
    SetProperty {
        object: Expr,
        property: String,
        property_span: Span,
        value: Expr,
    },
    SetIndex {
        root: String,
        root_span: Span,
        indices: Vec<Expr>,
        value: Expr,
    },
    Expression(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CatchClause {
    pub class_name: String,
    pub class_span: Span,
    pub variable: String,
    pub variable_span: Span,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub name_span: Span,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeSyntax,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub name_span: Span,
    pub abstract_class: bool,
    pub final_class: bool,
    pub parent: Option<NameRef>,
    pub interfaces: Vec<NameRef>,
    pub trait_uses: Vec<TraitUse>,
    pub properties: Vec<PropertyDecl>,
    pub methods: Vec<MethodDecl>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceDecl {
    pub name: String,
    pub name_span: Span,
    pub parent: Option<NameRef>,
    pub methods: Vec<MethodDecl>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TraitDecl {
    pub name: String,
    pub name_span: Span,
    pub trait_uses: Vec<TraitUse>,
    pub properties: Vec<PropertyDecl>,
    pub methods: Vec<MethodDecl>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NameRef {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TraitUse {
    pub traits: Vec<NameRef>,
    pub adaptations: Vec<TraitAdaptation>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TraitAdaptation {
    InsteadOf {
        trait_name: NameRef,
        method: NameRef,
        excluded: Vec<NameRef>,
        span: Span,
    },
    Alias {
        trait_name: NameRef,
        method: NameRef,
        visibility: Option<Visibility>,
        final_method: bool,
        alias: Option<NameRef>,
        span: Span,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyDecl {
    pub name: String,
    pub name_span: Span,
    pub visibility: Visibility,
    pub ty: TypeSyntax,
    pub initializer: Option<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MethodDecl {
    pub function: FunctionDecl,
    pub visibility: Visibility,
    pub static_method: bool,
    pub abstract_method: bool,
    pub final_method: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub name_span: Span,
    pub ty: TypeSyntax,
    pub default: Option<Expr>,
    pub variadic: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeSyntax {
    pub kind: TypeSyntaxKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeSyntaxKind {
    Named {
        name: String,
        arguments: Vec<TypeSyntax>,
    },
    Nullable(Box<TypeSyntax>),
    Union(Vec<TypeSyntax>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,
    String(Vec<u8>),
    Variable(String),
    Name(String),
    Vector(Vec<Expr>),
    Map(Vec<MapEntry>),
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Argument>,
    },
    Index {
        collection: Box<Expr>,
        index: Box<Expr>,
    },
    New {
        class_name: String,
        class_span: Span,
        arguments: Vec<Argument>,
    },
    Property {
        object: Box<Expr>,
        name: String,
        name_span: Span,
    },
    MethodCall {
        object: Box<Expr>,
        name: String,
        name_span: Span,
        arguments: Vec<Argument>,
    },
    StaticCall {
        target: ScopeTarget,
        class_span: Span,
        name: String,
        name_span: Span,
        arguments: Vec<Argument>,
    },
    ClassConstant {
        class_name: String,
        class_span: Span,
        name: String,
        name_span: Span,
    },
    InstanceOf {
        value: Box<Expr>,
        class_name: String,
        class_span: Span,
    },
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeTarget {
    Named(String),
    SelfType,
    Parent,
    Static,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Argument {
    pub name: Option<String>,
    pub name_span: Option<Span>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchArm {
    pub conditions: Vec<Expr>,
    pub value: Expr,
    pub default: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MapEntry {
    pub key: Expr,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Concatenate,
    Equal,
    StrictEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Coalesce,
}
