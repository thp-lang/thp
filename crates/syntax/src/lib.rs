//! THP source loading, lexing, spanned AST, and parsing.

#![allow(clippy::too_many_lines)]

mod ast;
mod lexer;
mod parser;
mod token;

pub use ast::{
    Argument, BinaryOp, Block, CatchClause, ClassDecl, Expr, ExprKind, ForClause, ForClauseKind,
    FunctionDecl, InterfaceDecl, LoopBinding, MapEntry, MatchArm, MethodDecl, NameRef,
    NamespaceDecl, Parameter, Program, PropertyDecl, QualifiedName, ScopeTarget, Stmt, StmtKind,
    TraitAdaptation, TraitDecl, TraitUse, TypeSyntax, TypeSyntaxKind, UnaryOp, UseDecl, UseKind,
    Visibility,
};
pub use lexer::{LexOutput, lex};
pub use parser::{ParseOutput, parse, parse_tokens};
pub use token::{Token, TokenKind};
