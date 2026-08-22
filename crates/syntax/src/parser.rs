use std::collections::BTreeSet;

use thp_diagnostics::{Diagnostic, SourceFile, Span};

use crate::{
    Argument, BinaryOp, Block, CatchClause, ClassDecl, Expr, ExprKind, ForClause, ForClauseKind,
    FunctionDecl, InterfaceDecl, LexOutput, LoopBinding, MapEntry, MatchArm, MethodDecl, NameRef,
    NamespaceDecl, Parameter, Program, PropertyDecl, QualifiedName, ScopeTarget, Stmt, StmtKind,
    Token, TokenKind, TraitAdaptation, TraitDecl, TraitUse, TypeSyntax, TypeSyntaxKind, UnaryOp,
    UseDecl, UseKind, Visibility, lex,
};

type ParsedMembers = (Vec<TraitUse>, Vec<PropertyDecl>, Vec<MethodDecl>, Span);

#[derive(Clone, Debug)]
pub struct ParseOutput {
    pub program: Program,
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(source: &SourceFile) -> ParseOutput {
    let LexOutput {
        tokens,
        diagnostics,
    } = lex(source);
    parse_tokens(source, tokens, diagnostics)
}

/// Parses a previously lexed token stream. This entry point lets the compiler
/// measure lexing and parsing independently without tokenizing twice.
pub fn parse_tokens(
    source: &SourceFile,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
) -> ParseOutput {
    let mut parser = Parser {
        source: source.text(),
        tokens: &tokens,
        current: 0,
        diagnostics,
    };
    let program = parser.parse_program();
    let diagnostics = std::mem::take(&mut parser.diagnostics);
    drop(parser);
    ParseOutput {
        program,
        tokens,
        diagnostics,
    }
}

struct Parser<'source, 'tokens> {
    source: &'source str,
    tokens: &'tokens [Token],
    current: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser<'_, '_> {
    fn parse_program(&mut self) -> Program {
        let start = self.current().span;
        if !self.consume(TokenKind::OpenTag) {
            self.error_current("P0001", "a THP source file must begin with `<?thp`");
        }
        let namespace = if self.at(TokenKind::Namespace) {
            self.parse_namespace_declaration()
        } else {
            None
        };
        let mut imports = Vec::new();
        while self.at(TokenKind::Use) {
            if let Some(import) = self.parse_use_declaration() {
                imports.push(import);
            } else {
                self.synchronize();
            }
        }
        let mut type_aliases = BTreeSet::new();
        let mut function_aliases = BTreeSet::new();
        for import in &imports {
            let aliases = match import.kind {
                UseKind::Type => &mut type_aliases,
                UseKind::Function => &mut function_aliases,
            };
            if !aliases.insert(import.alias.clone()) {
                self.error_span(
                    "P0013",
                    import.alias_span,
                    format!("duplicate import alias `{}`", import.alias),
                );
            }
        }
        let mut statements = Vec::new();
        while !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Namespace) {
                self.error_current(
                    "P0003",
                    "a file may contain only one namespace declaration immediately after `<?thp`",
                );
                self.advance();
                self.synchronize();
                continue;
            }
            if self.at(TokenKind::Use) {
                self.error_current(
                    "P0004",
                    "imports must appear before every top-level declaration or statement",
                );
                self.advance();
                self.synchronize();
                continue;
            }
            match self.parse_statement() {
                Some(statement) => statements.push(statement),
                None => self.synchronize(),
            }
        }
        Program {
            namespace,
            imports,
            statements,
            span: start.join(self.current().span),
        }
    }

    fn parse_namespace_declaration(&mut self) -> Option<NamespaceDecl> {
        let start = self.advance().span;
        let name = self.parse_qualified_name(false, "P0005", "expected a namespace name")?;
        if self.at(TokenKind::LBrace) {
            self.error_current(
                "P0006",
                "bracketed namespace declarations are not supported; use `namespace Name;`",
            );
            return None;
        }
        let end = self.expect(
            TokenKind::Semicolon,
            "P0007",
            "expected `;` after the namespace declaration",
        )?;
        Some(NamespaceDecl {
            span: start.join(end.span),
            name,
        })
    }

    fn parse_use_declaration(&mut self) -> Option<UseDecl> {
        let start = self.advance().span;
        if self.consume(TokenKind::Const) {
            self.error_span(
                "P0010",
                self.previous().span,
                "`use const` is not supported by THP modules",
            );
            return None;
        }
        let kind = if self.consume(TokenKind::Function) {
            UseKind::Function
        } else {
            UseKind::Type
        };
        let target = self.parse_qualified_name(true, "P0008", "expected an imported name")?;
        if self.at(TokenKind::Comma) || self.at(TokenKind::LBrace) {
            self.error_current(
                "P0011",
                "comma and group imports are not supported; write one `use` declaration per name",
            );
            return None;
        }
        let (alias, alias_span) = if self.consume(TokenKind::As) {
            let alias = self.expect(TokenKind::Identifier, "P0009", "expected an import alias")?;
            (self.text(alias.span).to_owned(), alias.span)
        } else {
            (target.last()?.to_owned(), target.span)
        };
        let end = self.expect(
            TokenKind::Semicolon,
            "P0012",
            "expected `;` after the import",
        )?;
        Some(UseDecl {
            kind,
            target,
            alias,
            alias_span,
            span: start.join(end.span),
        })
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.current().kind {
            TokenKind::Function => self.parse_function(),
            TokenKind::Class | TokenKind::Final | TokenKind::Abstract => self.parse_class(),
            TokenKind::Interface => self.parse_interface(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Echo => self.parse_echo(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Throw => self.parse_throw(),
            TokenKind::Try => self.parse_try(),
            TokenKind::Using => self.parse_using(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Foreach => self.parse_foreach(),
            TokenKind::Break => self.parse_loop_transfer(true),
            TokenKind::Continue => self.parse_loop_transfer(false),
            TokenKind::LBrace => {
                let start = self.advance().span;
                let (body, end) = self.parse_block_after_open()?;
                Some(Stmt {
                    kind: StmtKind::Block(body),
                    span: start.join(end),
                })
            }
            TokenKind::Variable
                if matches!(self.peek_kind(1), TokenKind::Colon | TokenKind::Equal) =>
            {
                self.parse_assignment()
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_function(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let (declaration, bodyless, end) = self.parse_function_after_keyword(start, true, false)?;
        debug_assert!(!bodyless);
        Some(Stmt {
            kind: StmtKind::Function(declaration),
            span: start.join(end),
        })
    }

    fn parse_function_after_keyword(
        &mut self,
        start: Span,
        require_return_type: bool,
        allow_bodyless: bool,
    ) -> Option<(FunctionDecl, bool, Span)> {
        let name = self.expect(TokenKind::Identifier, "P0101", "expected a function name")?;
        self.expect(
            TokenKind::LParen,
            "P0102",
            "expected `(` after the function name",
        )?;
        let mut parameters = Vec::new();
        if !self.at(TokenKind::RParen) {
            loop {
                let ty = self.parse_type()?;
                let variadic = self.consume(TokenKind::Ellipsis);
                let variable = self.expect(
                    TokenKind::Variable,
                    "P0103",
                    "expected a parameter variable after its type",
                )?;
                let default = if self.consume(TokenKind::Equal) {
                    Some(self.parse_expression(0)?)
                } else {
                    None
                };
                let span = default
                    .as_ref()
                    .map_or(ty.span.join(variable.span), |value| {
                        ty.span.join(value.span)
                    });
                parameters.push(Parameter {
                    name: self.text(variable.span)[1..].to_owned(),
                    name_span: variable.span,
                    default,
                    variadic,
                    span,
                    ty,
                });
                if !self.consume(TokenKind::Comma) {
                    break;
                }
                if self.at(TokenKind::RParen) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen, "P0104", "expected `)` after parameters")?;
        let function_name = self.text(name.span).to_owned();
        let return_type = if self.consume(TokenKind::Colon) {
            self.parse_type()?
        } else if !require_return_type && function_name == "__construct" {
            TypeSyntax {
                kind: TypeSyntaxKind::Named {
                    name: "void".to_owned(),
                    arguments: Vec::new(),
                },
                span: name.span,
            }
        } else {
            self.error_current("P0105", "expected a declared function return type");
            return None;
        };
        let (body, bodyless, end) = if allow_bodyless && self.consume(TokenKind::Semicolon) {
            (Vec::new(), true, self.previous().span)
        } else {
            let open = self.expect(TokenKind::LBrace, "P0106", "expected a function body")?;
            let (body, end) = self.parse_block_after_open()?;
            let _span = start.join(open.span).join(end);
            (body, false, end)
        };
        Some((
            FunctionDecl {
                name: function_name,
                name_span: name.span,
                parameters,
                return_type,
                body,
            },
            bodyless,
            end,
        ))
    }

    fn parse_class(&mut self) -> Option<Stmt> {
        let start = self.current().span;
        let mut final_class = false;
        let mut abstract_class = false;
        while matches!(self.current().kind, TokenKind::Final | TokenKind::Abstract) {
            match self.advance().kind {
                TokenKind::Final => final_class = true,
                TokenKind::Abstract => abstract_class = true,
                _ => unreachable!(),
            }
        }
        if final_class && abstract_class {
            self.error_span(
                "P0159",
                start,
                "a class cannot be both `abstract` and `final`",
            );
        }
        self.expect(TokenKind::Class, "P0150", "expected `class`")?;
        let name = self.expect(TokenKind::Identifier, "P0151", "expected a class name")?;
        let parent = if self.consume(TokenKind::Extends) {
            Some(self.parse_name_ref("P0152", "expected a parent class name")?)
        } else {
            None
        };
        let mut interfaces = Vec::new();
        if self.consume(TokenKind::Implements) {
            loop {
                interfaces
                    .push(self.parse_name_ref("P0153", "expected an implemented interface name")?);
                if !self.consume(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::LBrace, "P0154", "expected a class body")?;
        let (trait_uses, properties, methods, end) = self.parse_members(false)?;
        Some(Stmt {
            kind: StmtKind::Class(ClassDecl {
                name: self.text(name.span).to_owned(),
                name_span: name.span,
                abstract_class,
                final_class,
                parent,
                interfaces,
                trait_uses,
                properties,
                methods,
            }),
            span: start.join(end),
        })
    }

    fn parse_interface(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let name = self.expect(TokenKind::Identifier, "P0160", "expected an interface name")?;
        let parent = if self.consume(TokenKind::Extends) {
            let parent = self.parse_name_ref("P0161", "expected a parent interface name")?;
            if self.consume(TokenKind::Comma) {
                self.error_span(
                    "P0162",
                    self.previous().span,
                    "an interface may extend at most one interface",
                );
                while self.at(TokenKind::Identifier) || self.at(TokenKind::NamespaceSeparator) {
                    let _ = self.parse_qualified_name(true, "P0161", "expected an interface name");
                    if !self.consume(TokenKind::Comma) {
                        break;
                    }
                }
            }
            Some(parent)
        } else {
            None
        };
        self.expect(TokenKind::LBrace, "P0163", "expected an interface body")?;
        let (trait_uses, properties, methods, end) = self.parse_members(true)?;
        debug_assert!(trait_uses.is_empty());
        debug_assert!(properties.is_empty());
        Some(Stmt {
            kind: StmtKind::Interface(InterfaceDecl {
                name: self.text(name.span).to_owned(),
                name_span: name.span,
                parent,
                methods,
            }),
            span: start.join(end),
        })
    }

    fn parse_trait(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let name = self.expect(TokenKind::Identifier, "P0170", "expected a trait name")?;
        self.expect(TokenKind::LBrace, "P0171", "expected a trait body")?;
        let (trait_uses, properties, methods, end) = self.parse_members(false)?;
        Some(Stmt {
            kind: StmtKind::Trait(TraitDecl {
                name: self.text(name.span).to_owned(),
                name_span: name.span,
                trait_uses,
                properties,
                methods,
            }),
            span: start.join(end),
        })
    }

    fn parse_members(&mut self, interface: bool) -> Option<ParsedMembers> {
        let mut trait_uses = Vec::new();
        let mut properties = Vec::new();
        let mut methods = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let member_start = self.current().span;
            if self.at(TokenKind::Use) {
                if interface {
                    self.error_current("P0164", "interfaces cannot use traits");
                    return None;
                }
                trait_uses.push(self.parse_trait_use()?);
                continue;
            }

            let mut visibility = None;
            let mut static_method = false;
            let mut abstract_method = false;
            let mut final_method = false;
            while matches!(
                self.current().kind,
                TokenKind::Public
                    | TokenKind::Protected
                    | TokenKind::Private
                    | TokenKind::Static
                    | TokenKind::Abstract
                    | TokenKind::Final
            ) {
                let modifier = self.advance();
                match modifier.kind {
                    TokenKind::Public => visibility = Some(Visibility::Public),
                    TokenKind::Protected => visibility = Some(Visibility::Protected),
                    TokenKind::Private => visibility = Some(Visibility::Private),
                    TokenKind::Static => static_method = true,
                    TokenKind::Abstract => abstract_method = true,
                    TokenKind::Final => final_method = true,
                    _ => unreachable!(),
                }
            }
            let visibility = visibility.unwrap_or(Visibility::Public);
            if self.consume(TokenKind::Function) {
                let (function, bodyless, end) =
                    self.parse_function_after_keyword(member_start, false, true)?;
                if interface && !bodyless {
                    self.error_span(
                        "P0165",
                        member_start.join(end),
                        "interface methods must end in `;` and cannot have a body",
                    );
                }
                if interface && visibility != Visibility::Public {
                    self.error_span("P0166", member_start, "interface methods must be public");
                }
                if abstract_method && !bodyless {
                    self.error_span(
                        "P0172",
                        member_start.join(end),
                        "abstract methods must end in `;` and cannot have a body",
                    );
                }
                if bodyless && !interface && !abstract_method {
                    self.error_span(
                        "P0173",
                        member_start.join(end),
                        "a bodyless method must be declared `abstract`",
                    );
                }
                methods.push(MethodDecl {
                    function,
                    visibility,
                    static_method,
                    abstract_method: interface || abstract_method,
                    final_method,
                    span: member_start.join(end),
                });
            } else {
                if interface {
                    self.error_current("P0167", "interfaces may contain only methods");
                    return None;
                }
                if static_method {
                    self.error_current("P0155", "static properties are not supported");
                    return None;
                }
                if abstract_method || final_method {
                    self.error_current("P0174", "abstract and final properties are not supported");
                    return None;
                }
                let ty = self.parse_type()?;
                let variable = self.expect(
                    TokenKind::Variable,
                    "P0156",
                    "expected a property variable after its type",
                )?;
                let initializer = if self.consume(TokenKind::Equal) {
                    let initializer = self.parse_expression(0)?;
                    if !is_property_initializer(&initializer) {
                        self.error_span(
                            "P0159",
                            initializer.span,
                            "property defaults must be constant expressions",
                        );
                    }
                    Some(initializer)
                } else {
                    None
                };
                let end = self.expect(
                    TokenKind::Semicolon,
                    "P0157",
                    "expected `;` after property declaration",
                )?;
                properties.push(PropertyDecl {
                    name: self.text(variable.span)[1..].to_owned(),
                    name_span: variable.span,
                    visibility,
                    span: member_start.join(end.span),
                    ty,
                    initializer,
                });
            }
        }
        let end = self.expect(
            TokenKind::RBrace,
            "P0158",
            "expected `}` after nominal type body",
        )?;
        Some((trait_uses, properties, methods, end.span))
    }

    fn parse_trait_use(&mut self) -> Option<TraitUse> {
        let start = self.advance().span;
        let mut traits = Vec::new();
        loop {
            traits.push(self.parse_name_ref("P0180", "expected a trait name")?);
            if !self.consume(TokenKind::Comma) {
                break;
            }
        }
        if self.consume(TokenKind::Semicolon) {
            return Some(TraitUse {
                traits,
                adaptations: Vec::new(),
                span: start.join(self.previous().span),
            });
        }
        self.expect(
            TokenKind::LBrace,
            "P0181",
            "expected `;` or a trait adaptation block",
        )?;
        let mut adaptations = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let adaptation_start = self.current().span;
            let trait_name = self.parse_name_ref("P0182", "expected the source trait name")?;
            self.expect(
                TokenKind::DoubleColon,
                "P0183",
                "expected `::` after the source trait",
            )?;
            let method = self.expect(
                TokenKind::Identifier,
                "P0184",
                "expected a trait method name",
            )?;
            let method = self.name_ref(method);
            if self.consume(TokenKind::InsteadOf) {
                let mut excluded = Vec::new();
                loop {
                    excluded.push(self.parse_name_ref("P0185", "expected an excluded trait name")?);
                    if !self.consume(TokenKind::Comma) {
                        break;
                    }
                }
                let end = self.expect(
                    TokenKind::Semicolon,
                    "P0186",
                    "expected `;` after `insteadof` adaptation",
                )?;
                adaptations.push(TraitAdaptation::InsteadOf {
                    trait_name,
                    method,
                    excluded,
                    span: adaptation_start.join(end.span),
                });
            } else {
                self.expect(TokenKind::As, "P0187", "expected `as` or `insteadof`")?;
                let visibility = match self.current().kind {
                    TokenKind::Public => {
                        self.advance();
                        Some(Visibility::Public)
                    }
                    TokenKind::Protected => {
                        self.advance();
                        Some(Visibility::Protected)
                    }
                    TokenKind::Private => {
                        self.advance();
                        Some(Visibility::Private)
                    }
                    _ => None,
                };
                let final_method = self.consume(TokenKind::Final);
                let alias = self
                    .consume(TokenKind::Identifier)
                    .then(|| self.name_ref(self.previous()));
                if visibility.is_none() && !final_method && alias.is_none() {
                    self.error_current(
                        "P0188",
                        "`as` must change visibility/finality or add an alias",
                    );
                }
                let end = self.expect(
                    TokenKind::Semicolon,
                    "P0189",
                    "expected `;` after trait alias adaptation",
                )?;
                adaptations.push(TraitAdaptation::Alias {
                    trait_name,
                    method,
                    visibility,
                    final_method,
                    alias,
                    span: adaptation_start.join(end.span),
                });
            }
        }
        let end = self.expect(
            TokenKind::RBrace,
            "P0190",
            "expected `}` after trait adaptations",
        )?;
        Some(TraitUse {
            traits,
            adaptations,
            span: start.join(end.span),
        })
    }

    fn parse_assignment(&mut self) -> Option<Stmt> {
        let variable = self.advance();
        let annotation = if self.consume(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(
            TokenKind::Equal,
            "P0201",
            "expected `=` in variable assignment",
        )?;
        let value = self.parse_expression(0)?;
        let end = self.expect(
            TokenKind::Semicolon,
            "P0202",
            "expected `;` after variable assignment",
        )?;
        Some(Stmt {
            kind: StmtKind::Assign {
                name: self.text(variable.span)[1..].to_owned(),
                annotation,
                value,
            },
            span: variable.span.join(end.span),
        })
    }

    fn parse_echo(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let expression = self.parse_expression(0)?;
        if self.at(TokenKind::Comma) {
            self.error_current(
                "P0302",
                "`echo` accepts one expression; concatenate values with `.`",
            );
            return None;
        }
        let end = self.expect(TokenKind::Semicolon, "P0301", "expected `;` after echo")?;
        Some(Stmt {
            kind: StmtKind::Echo(expression),
            span: start.join(end.span),
        })
    }

    fn parse_return(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let value = if self.at(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression(0)?)
        };
        let end = self.expect(TokenKind::Semicolon, "P0401", "expected `;` after return")?;
        Some(Stmt {
            kind: StmtKind::Return(value),
            span: start.join(end.span),
        })
    }

    fn parse_try(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        self.expect(TokenKind::LBrace, "P0450", "expected a block after `try`")?;
        let (body, mut end) = self.parse_block_after_open()?;
        let mut catches = Vec::new();
        while self.consume(TokenKind::Catch) {
            let catch_start = self.previous().span;
            self.expect(TokenKind::LParen, "P0451", "expected `(` after `catch`")?;
            let class =
                self.parse_qualified_name(true, "P0452", "expected a throwable class name")?;
            let variable =
                self.expect(TokenKind::Variable, "P0453", "expected a catch variable")?;
            self.expect(
                TokenKind::RParen,
                "P0454",
                "expected `)` after catch parameter",
            )?;
            self.expect(TokenKind::LBrace, "P0455", "expected a catch block")?;
            let (catch_body, catch_end) = self.parse_block_after_open()?;
            catches.push(CatchClause {
                class_name: class.as_string(),
                class_span: class.span,
                variable: self.text(variable.span)[1..].to_owned(),
                variable_span: variable.span,
                body: catch_body,
                span: catch_start.join(catch_end),
            });
            end = catch_end;
        }
        let finally = if self.consume(TokenKind::Finally) {
            self.expect(TokenKind::LBrace, "P0457", "expected a `finally` block")?;
            let (body, finally_end) = self.parse_block_after_open()?;
            end = finally_end;
            Some(body)
        } else {
            None
        };
        if catches.is_empty() && finally.is_none() {
            self.error_current("P0456", "`try` requires `catch` or `finally`");
        }
        Some(Stmt {
            kind: StmtKind::Try {
                body,
                catches,
                finally,
            },
            span: start.join(end),
        })
    }

    fn parse_using(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        self.expect(TokenKind::LParen, "P0460", "expected `(` after `using`")?;
        let variable = self.expect(
            TokenKind::Variable,
            "P0461",
            "expected a variable declaration in `using`",
        )?;
        self.expect(
            TokenKind::Equal,
            "P0462",
            "expected `=` in `using` declaration",
        )?;
        let value = self.parse_expression(0)?;
        self.expect(
            TokenKind::RParen,
            "P0463",
            "expected `)` after `using` declaration",
        )?;
        self.expect(TokenKind::LBrace, "P0464", "expected a `using` body")?;
        let (body, end) = self.parse_block_after_open()?;
        Some(Stmt {
            kind: StmtKind::Using {
                variable: self.text(variable.span)[1..].to_owned(),
                variable_span: variable.span,
                value,
                body,
            },
            span: start.join(end),
        })
    }

    fn parse_throw(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let value = self.parse_expression(0)?;
        let end = self.expect(TokenKind::Semicolon, "P0450", "expected `;` after throw")?;
        Some(Stmt {
            kind: StmtKind::Throw(value),
            span: start.join(end.span),
        })
    }

    fn parse_if(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let mut branches = Vec::new();
        let condition = self.parse_parenthesized_condition("if")?;
        let open = self.expect(TokenKind::LBrace, "P0501", "expected an `if` block")?;
        let (body, mut end) = self.parse_block_after_open()?;
        branches.push((condition, body));
        while self.consume(TokenKind::ElseIf) {
            let condition = self.parse_parenthesized_condition("elseif")?;
            self.expect(TokenKind::LBrace, "P0502", "expected an `elseif` block")?;
            let (body, branch_end) = self.parse_block_after_open()?;
            end = branch_end;
            branches.push((condition, body));
        }
        let otherwise = if self.consume(TokenKind::Else) {
            self.expect(TokenKind::LBrace, "P0503", "expected an `else` block")?;
            let (body, branch_end) = self.parse_block_after_open()?;
            end = branch_end;
            Some(body)
        } else {
            None
        };
        Some(Stmt {
            kind: StmtKind::If {
                branches,
                otherwise,
            },
            span: start.join(open.span).join(end),
        })
    }

    fn parse_while(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let condition = self.parse_parenthesized_condition("while")?;
        self.expect(TokenKind::LBrace, "P0601", "expected a `while` block")?;
        let (body, end) = self.parse_block_after_open()?;
        Some(Stmt {
            kind: StmtKind::While { condition, body },
            span: start.join(end),
        })
    }

    fn parse_for(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        self.expect(TokenKind::LParen, "P0610", "expected `(` after `for`")?;
        let initializers = self.parse_for_clause_list(TokenKind::Semicolon)?;
        self.expect(
            TokenKind::Semicolon,
            "P0611",
            "expected `;` after `for` initializers",
        )?;
        let conditions = self.parse_for_clause_list(TokenKind::Semicolon)?;
        self.expect(
            TokenKind::Semicolon,
            "P0612",
            "expected `;` after `for` conditions",
        )?;
        let updates = self.parse_for_clause_list(TokenKind::RParen)?;
        self.expect(
            TokenKind::RParen,
            "P0613",
            "expected `)` after `for` clauses",
        )?;
        self.expect(TokenKind::LBrace, "P0614", "expected a `for` block")?;
        let (body, end) = self.parse_block_after_open()?;
        Some(Stmt {
            kind: StmtKind::For {
                initializers,
                conditions,
                updates,
                body,
            },
            span: start.join(end),
        })
    }

    fn parse_for_clause_list(&mut self, end: TokenKind) -> Option<Vec<ForClause>> {
        let mut clauses = Vec::new();
        if self.at(end) {
            return Some(clauses);
        }
        loop {
            clauses.push(self.parse_for_clause()?);
            if !self.consume(TokenKind::Comma) {
                break;
            }
            if self.at(end) {
                self.error_current("P0615", "expected a clause after `,`");
                return None;
            }
        }
        Some(clauses)
    }

    fn parse_for_clause(&mut self) -> Option<ForClause> {
        if self.at(TokenKind::Variable) && self.peek_kind(1) == TokenKind::Colon {
            let variable = self.advance();
            self.advance();
            let annotation = self.parse_type()?;
            self.expect(
                TokenKind::Equal,
                "P0616",
                "expected `=` in `for` assignment",
            )?;
            let value = self.parse_expression(0)?;
            return Some(ForClause {
                span: variable.span.join(value.span),
                kind: ForClauseKind::Assign {
                    name: self.text(variable.span)[1..].to_owned(),
                    name_span: variable.span,
                    annotation: Some(annotation),
                    value,
                },
            });
        }
        let expression = self.parse_expression(0)?;
        if !self.consume(TokenKind::Equal) {
            let span = expression.span;
            return Some(ForClause {
                kind: ForClauseKind::Expression(expression),
                span,
            });
        }
        let value = self.parse_expression(0)?;
        let span = expression.span.join(value.span);
        let kind = match expression.kind {
            ExprKind::Variable(name) => ForClauseKind::Assign {
                name,
                name_span: expression.span,
                annotation: None,
                value,
            },
            ExprKind::Property {
                object,
                name,
                name_span,
            } => ForClauseKind::SetProperty {
                object: *object,
                property: name,
                property_span: name_span,
                value,
            },
            _ => {
                let Some((root, root_span, indices)) = index_assignment_target(expression) else {
                    self.error_span(
                        "P0617",
                        span,
                        "`for` assignment target must be a variable, property, or variable-rooted collection element",
                    );
                    return None;
                };
                ForClauseKind::SetIndex {
                    root,
                    root_span,
                    indices,
                    value,
                }
            }
        };
        Some(ForClause { kind, span })
    }

    fn parse_foreach(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        self.expect(TokenKind::LParen, "P0620", "expected `(` after `foreach`")?;
        let source = self.parse_expression(0)?;
        self.expect(TokenKind::As, "P0621", "expected `as` in `foreach`")?;
        let first = self.expect(
            TokenKind::Variable,
            "P0622",
            "expected a value variable after `as`",
        )?;
        let first_binding = LoopBinding {
            name: self.text(first.span)[1..].to_owned(),
            span: first.span,
        };
        let (key, value) = if self.consume(TokenKind::FatArrow) {
            let value = self.expect(
                TokenKind::Variable,
                "P0623",
                "expected a value variable after `=>`",
            )?;
            (
                Some(first_binding),
                LoopBinding {
                    name: self.text(value.span)[1..].to_owned(),
                    span: value.span,
                },
            )
        } else {
            (None, first_binding)
        };
        self.expect(TokenKind::RParen, "P0624", "expected `)` after `foreach`")?;
        self.expect(TokenKind::LBrace, "P0625", "expected a `foreach` block")?;
        let (body, end) = self.parse_block_after_open()?;
        Some(Stmt {
            kind: StmtKind::Foreach {
                source,
                key,
                value,
                body,
            },
            span: start.join(end),
        })
    }

    fn parse_loop_transfer(&mut self, is_break: bool) -> Option<Stmt> {
        let start = self.advance().span;
        let end = self.expect(
            TokenKind::Semicolon,
            if is_break { "P0630" } else { "P0631" },
            if is_break {
                "expected `;` after `break`; numeric levels are not supported"
            } else {
                "expected `;` after `continue`; numeric levels are not supported"
            },
        )?;
        Some(Stmt {
            kind: if is_break {
                StmtKind::Break
            } else {
                StmtKind::Continue
            },
            span: start.join(end.span),
        })
    }

    fn parse_parenthesized_condition(&mut self, keyword: &str) -> Option<Expr> {
        self.expect(
            TokenKind::LParen,
            "P0504",
            format!("expected `(` after `{keyword}`"),
        )?;
        let condition = self.parse_expression(0)?;
        self.expect(TokenKind::RParen, "P0505", "expected `)` after condition")?;
        Some(condition)
    }

    fn parse_block_after_open(&mut self) -> Option<(Block, Span)> {
        let mut statements = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            match self.parse_statement() {
                Some(statement) => statements.push(statement),
                None => self.synchronize(),
            }
        }
        let end = self.expect(TokenKind::RBrace, "P0701", "expected `}` to close block")?;
        Some((statements, end.span))
    }

    fn parse_expression_statement(&mut self) -> Option<Stmt> {
        let expression = self.parse_expression(0)?;
        if self.consume(TokenKind::Equal) {
            let value = self.parse_expression(0)?;
            let end = self.expect(
                TokenKind::Semicolon,
                "P0802",
                "expected `;` after assignment",
            )?;
            let assignment_span = expression.span.join(end.span);
            return if let ExprKind::Property {
                object,
                name,
                name_span,
            } = expression.kind
            {
                Some(Stmt {
                    kind: StmtKind::SetProperty {
                        object: *object,
                        property: name,
                        property_span: name_span,
                        value,
                    },
                    span: assignment_span,
                })
            } else {
                let Some((root, root_span, indices)) = index_assignment_target(expression) else {
                    self.error_span(
                        "P0803",
                        assignment_span,
                        "only variables, object properties, and variable-rooted collection elements are assignable",
                    );
                    return None;
                };
                Some(Stmt {
                    kind: StmtKind::SetIndex {
                        root,
                        root_span,
                        indices,
                        value,
                    },
                    span: assignment_span,
                })
            };
        }
        let end = self.expect(
            TokenKind::Semicolon,
            "P0801",
            "expected `;` after expression",
        )?;
        let span = expression.span.join(end.span);
        Some(Stmt {
            kind: StmtKind::Expression(expression),
            span,
        })
    }

    fn parse_type(&mut self) -> Option<TypeSyntax> {
        let mut members = vec![self.parse_atomic_type()?];
        while self.consume(TokenKind::Pipe) {
            members.push(self.parse_atomic_type()?);
        }
        if members.len() == 1 {
            members.pop()
        } else {
            let span = members
                .first()
                .expect("union has a first type")
                .span
                .join(members.last().expect("union has a last type").span);
            Some(TypeSyntax {
                kind: TypeSyntaxKind::Union(members),
                span,
            })
        }
    }

    fn parse_atomic_type(&mut self) -> Option<TypeSyntax> {
        if self.consume(TokenKind::Question) {
            let question = self.previous().span;
            let inner = self.parse_atomic_type()?;
            return Some(TypeSyntax {
                span: question.join(inner.span),
                kind: TypeSyntaxKind::Nullable(Box::new(inner)),
            });
        }
        let name = self.parse_qualified_name(true, "P0901", "expected a type name")?;
        let mut arguments = Vec::new();
        let mut end = name.span;
        if self.consume(TokenKind::Less) {
            loop {
                arguments.push(self.parse_type()?);
                if !self.consume(TokenKind::Comma) {
                    break;
                }
            }
            end = self
                .expect(
                    TokenKind::Greater,
                    "P0902",
                    "expected `>` after generic type arguments",
                )?
                .span;
        }
        Some(TypeSyntax {
            kind: TypeSyntaxKind::Named {
                name: name.as_string(),
                arguments,
            },
            span: name.span.join(end),
        })
    }

    fn parse_expression(&mut self, minimum_precedence: u8) -> Option<Expr> {
        let mut left = self.parse_prefix()?;
        loop {
            if self.at(TokenKind::LParen) {
                if 12 < minimum_precedence {
                    break;
                }
                left = self.finish_call(left)?;
                continue;
            }
            if self.at(TokenKind::LBracket) {
                if 12 < minimum_precedence {
                    break;
                }
                let open = self.advance();
                let index = self.parse_expression(0)?;
                let end = self.expect(TokenKind::RBracket, "P1001", "expected `]` after index")?;
                let span = left.span.join(open.span).join(end.span);
                left = Expr {
                    kind: ExprKind::Index {
                        collection: Box::new(left),
                        index: Box::new(index),
                    },
                    span,
                };
                continue;
            }
            if self.at(TokenKind::Arrow) {
                if 12 < minimum_precedence {
                    break;
                }
                self.advance();
                let name = self.expect(
                    TokenKind::Identifier,
                    "P1002",
                    "expected a property or method name after `->`",
                )?;
                let name_text = self.text(name.span).to_owned();
                if self.consume(TokenKind::LParen) {
                    let (arguments, end) = self.parse_arguments_after_open()?;
                    let span = left.span.join(end);
                    left = Expr {
                        kind: ExprKind::MethodCall {
                            object: Box::new(left),
                            name: name_text,
                            name_span: name.span,
                            arguments,
                        },
                        span,
                    };
                } else {
                    let span = left.span.join(name.span);
                    left = Expr {
                        kind: ExprKind::Property {
                            object: Box::new(left),
                            name: name_text,
                            name_span: name.span,
                        },
                        span,
                    };
                }
                continue;
            }
            if self.at(TokenKind::DoubleColon) {
                if 12 < minimum_precedence {
                    break;
                }
                let ExprKind::Name(class_name) = left.kind else {
                    self.error_span("P1003", left.span, "static access requires a class name");
                    return None;
                };
                let class_span = left.span;
                let target = match class_name.as_str() {
                    "self" => ScopeTarget::SelfType,
                    "parent" => ScopeTarget::Parent,
                    "static" => ScopeTarget::Static,
                    _ => ScopeTarget::Named(class_name.clone()),
                };
                self.advance();
                let name = self.expect(
                    TokenKind::Identifier,
                    "P1004",
                    "expected a member name after `::`",
                )?;
                if self.consume(TokenKind::LParen) {
                    let (arguments, end) = self.parse_arguments_after_open()?;
                    left = Expr {
                        kind: ExprKind::StaticCall {
                            target,
                            class_span,
                            name: self.text(name.span).to_owned(),
                            name_span: name.span,
                            arguments,
                        },
                        span: class_span.join(end),
                    };
                } else {
                    let ScopeTarget::Named(class_name) = target else {
                        self.error_span(
                            "P1005",
                            class_span,
                            "scoped class constants are not implemented",
                        );
                        return None;
                    };
                    left = Expr {
                        kind: ExprKind::ClassConstant {
                            class_name,
                            class_span,
                            name: self.text(name.span).to_owned(),
                            name_span: name.span,
                        },
                        span: class_span.join(name.span),
                    };
                }
                continue;
            }
            if self.at(TokenKind::InstanceOf) {
                let precedence = 5;
                if precedence < minimum_precedence {
                    break;
                }
                self.advance();
                let class = self.parse_qualified_name(
                    true,
                    "P1006",
                    "expected a type name after `instanceof`",
                )?;
                let span = left.span.join(class.span);
                left = Expr {
                    kind: ExprKind::InstanceOf {
                        value: Box::new(left),
                        class_name: class.as_string(),
                        class_span: class.span,
                    },
                    span,
                };
                continue;
            }
            let Some((precedence, op)) = binary_operator(self.current().kind) else {
                break;
            };
            if precedence < minimum_precedence {
                break;
            }
            self.advance();
            let right = self.parse_expression(precedence + 1)?;
            let span = left.span.join(right.span);
            left = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            };
        }
        Some(left)
    }

    fn parse_prefix(&mut self) -> Option<Expr> {
        let token = self.advance();
        let mut expression = match token.kind {
            TokenKind::Integer => {
                let value = self.text(token.span).parse::<i64>().map_err(|_| {
                    self.diagnostics.push(Diagnostic::error(
                        "parsing",
                        "P1101",
                        token.span,
                        "integer literal is outside the signed 64-bit range",
                    ));
                });
                Expr {
                    kind: ExprKind::Integer(value.ok()?),
                    span: token.span,
                }
            }
            TokenKind::Float => {
                let value = self.text(token.span).parse::<f64>().map_err(|_| {
                    self.diagnostics.push(Diagnostic::error(
                        "parsing",
                        "P1102",
                        token.span,
                        "invalid floating-point literal",
                    ));
                });
                Expr {
                    kind: ExprKind::Float(value.ok()?),
                    span: token.span,
                }
            }
            TokenKind::True | TokenKind::False => Expr {
                kind: ExprKind::Bool(token.kind == TokenKind::True),
                span: token.span,
            },
            TokenKind::Null => Expr {
                kind: ExprKind::Null,
                span: token.span,
            },
            TokenKind::String => Expr {
                kind: ExprKind::String(self.decode_string(token.span)?),
                span: token.span,
            },
            TokenKind::New => {
                let class =
                    self.parse_qualified_name(true, "P1104", "expected a class name after `new`")?;
                self.expect(TokenKind::LParen, "P1105", "expected constructor arguments")?;
                let (arguments, end) = self.parse_arguments_after_open()?;
                Expr {
                    kind: ExprKind::New {
                        class_name: class.as_string(),
                        class_span: class.span,
                        arguments,
                    },
                    span: token.span.join(end),
                }
            }
            TokenKind::Variable => Expr {
                kind: ExprKind::Variable(self.text(token.span)[1..].to_owned()),
                span: token.span,
            },
            TokenKind::Identifier | TokenKind::NamespaceSeparator => {
                self.current = self.current.saturating_sub(1);
                let name = self.parse_qualified_name(true, "P1106", "expected a qualified name")?;
                Expr {
                    kind: ExprKind::Name(name.as_string()),
                    span: name.span,
                }
            }
            TokenKind::Static => Expr {
                kind: ExprKind::Name(self.text(token.span).to_owned()),
                span: token.span,
            },
            TokenKind::Minus | TokenKind::Bang => {
                let operand = self.parse_expression(11)?;
                let span = token.span.join(operand.span);
                Expr {
                    kind: ExprKind::Unary {
                        op: if token.kind == TokenKind::Minus {
                            UnaryOp::Negate
                        } else {
                            UnaryOp::Not
                        },
                        operand: Box::new(operand),
                    },
                    span,
                }
            }
            TokenKind::LParen => {
                let inner = self.parse_expression(0)?;
                let end =
                    self.expect(TokenKind::RParen, "P1103", "expected `)` after expression")?;
                Expr {
                    span: token.span.join(end.span),
                    ..inner
                }
            }
            TokenKind::LBracket => self.parse_vector(token.span)?,
            TokenKind::LBrace => self.parse_map(token.span)?,
            TokenKind::Match => self.parse_match(token.span)?,
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "parsing",
                    "P1100",
                    token.span,
                    format!("expected an expression, found {}", token.kind.description()),
                ));
                return None;
            }
        };
        while self.at(TokenKind::LParen) {
            expression = self.finish_call(expression)?;
        }
        Some(expression)
    }

    fn parse_vector(&mut self, start: Span) -> Option<Expr> {
        let mut values = Vec::new();
        if !self.at(TokenKind::RBracket) {
            loop {
                values.push(self.parse_expression(0)?);
                if !self.consume(TokenKind::Comma) {
                    break;
                }
                if self.at(TokenKind::RBracket) {
                    break;
                }
            }
        }
        let end = self.expect(
            TokenKind::RBracket,
            "P1201",
            "expected `]` after vector literal",
        )?;
        Some(Expr {
            kind: ExprKind::Vector(values),
            span: start.join(end.span),
        })
    }

    fn parse_map(&mut self, start: Span) -> Option<Expr> {
        let mut entries = Vec::new();
        if !self.at(TokenKind::RBrace) {
            loop {
                let key = self.parse_expression(0)?;
                self.expect(
                    TokenKind::FatArrow,
                    "P1202",
                    "expected `=>` between a map key and value",
                )?;
                let value = self.parse_expression(0)?;
                let span = key.span.join(value.span);
                entries.push(MapEntry { key, value, span });
                if !self.consume(TokenKind::Comma) {
                    break;
                }
                if self.at(TokenKind::RBrace) {
                    break;
                }
            }
        }
        let end = self.expect(TokenKind::RBrace, "P1203", "expected `}` after map literal")?;
        Some(Expr {
            kind: ExprKind::Map(entries),
            span: start.join(end.span),
        })
    }

    fn finish_call(&mut self, callee: Expr) -> Option<Expr> {
        self.expect(TokenKind::LParen, "P1301", "expected `(`")?;
        let (arguments, end) = self.parse_arguments_after_open()?;
        let span = callee.span.join(end);
        Some(Expr {
            kind: ExprKind::Call {
                callee: Box::new(callee),
                arguments,
            },
            span,
        })
    }

    fn parse_arguments_after_open(&mut self) -> Option<(Vec<Argument>, Span)> {
        let mut arguments = Vec::new();
        if !self.at(TokenKind::RParen) {
            loop {
                if self.consume(TokenKind::Ellipsis) {
                    self.error_span(
                        "P1303",
                        self.previous().span,
                        "call-site argument unpacking is not supported",
                    );
                }
                let (name, name_span) =
                    if self.at(TokenKind::Identifier) && self.peek_kind(1) == TokenKind::Colon {
                        let name = self.advance();
                        self.advance();
                        (Some(self.text(name.span).to_owned()), Some(name.span))
                    } else {
                        (None, None)
                    };
                let value = self.parse_expression(0)?;
                let span = name_span.map_or(value.span, |name_span| name_span.join(value.span));
                arguments.push(Argument {
                    name,
                    name_span,
                    value,
                    span,
                });
                if !self.consume(TokenKind::Comma) {
                    break;
                }
                if self.at(TokenKind::RParen) {
                    break;
                }
            }
        }
        let end = self.expect(TokenKind::RParen, "P1302", "expected `)` after arguments")?;
        Some((arguments, end.span))
    }

    fn parse_match(&mut self, start: Span) -> Option<Expr> {
        self.expect(TokenKind::LParen, "P1401", "expected `(` after `match`")?;
        let subject = self.parse_expression(0)?;
        self.expect(
            TokenKind::RParen,
            "P1402",
            "expected `)` after match subject",
        )?;
        self.expect(TokenKind::LBrace, "P1403", "expected `{` before match arms")?;
        let mut arms = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let arm_start = self.current().span;
            let (default, conditions) = if self.consume(TokenKind::Default) {
                (true, Vec::new())
            } else {
                let mut conditions = vec![self.parse_expression(0)?];
                while self.consume(TokenKind::Comma) {
                    if self.at(TokenKind::FatArrow) {
                        self.error_current("P1404", "expected a match condition after `,`");
                        return None;
                    }
                    conditions.push(self.parse_expression(0)?);
                }
                (false, conditions)
            };
            self.expect(TokenKind::FatArrow, "P1405", "expected `=>` in match arm")?;
            let value = self.parse_expression(0)?;
            let span = arm_start.join(value.span);
            arms.push(MatchArm {
                conditions,
                value,
                default,
                span,
            });
            if !self.consume(TokenKind::Comma) {
                break;
            }
        }
        let end = self.expect(TokenKind::RBrace, "P1406", "expected `}` after match arms")?;
        Some(Expr {
            kind: ExprKind::Match {
                subject: Box::new(subject),
                arms,
            },
            span: start.join(end.span),
        })
    }

    fn decode_string(&mut self, span: Span) -> Option<Vec<u8>> {
        let text = self.text(span);
        let quote = text.as_bytes()[0];
        let content = &text[1..text.len() - 1];
        let mut decoded = Vec::with_capacity(content.len());
        let bytes = content.as_bytes();
        let mut index = 0;
        while let Some(byte) = bytes.get(index).copied() {
            if byte != b'\\' {
                decoded.push(byte);
                index += 1;
                continue;
            }
            index += 1;
            let Some(escaped) = bytes.get(index).copied() else {
                self.error_span("P1401", span, "incomplete string escape");
                return None;
            };
            index += 1;
            match escaped {
                b'n' if quote == b'"' => decoded.push(b'\n'),
                b'r' if quote == b'"' => decoded.push(b'\r'),
                b't' if quote == b'"' => decoded.push(b'\t'),
                b'0' if quote == b'"' => decoded.push(0),
                b'x' if quote == b'"' => {
                    let Some(hex) = bytes.get(index..index + 2) else {
                        self.error_span("P1403", span, "`\\x` requires two hexadecimal digits");
                        return None;
                    };
                    let Ok(hex) = std::str::from_utf8(hex) else {
                        self.error_span("P1403", span, "`\\x` requires two hexadecimal digits");
                        return None;
                    };
                    let Ok(value) = u8::from_str_radix(hex, 16) else {
                        self.error_span("P1403", span, "`\\x` requires two hexadecimal digits");
                        return None;
                    };
                    decoded.push(value);
                    index += 2;
                }
                b'\\' => decoded.push(b'\\'),
                b'\'' if quote == b'\'' => decoded.push(b'\''),
                b'"' if quote == b'"' => decoded.push(b'"'),
                _ => {
                    self.error_span("P1402", span, "unsupported string escape");
                    return None;
                }
            }
        }
        Some(decoded)
    }

    fn parse_qualified_name(
        &mut self,
        allow_absolute: bool,
        code: &'static str,
        message: &'static str,
    ) -> Option<QualifiedName> {
        let absolute = self.consume(TokenKind::NamespaceSeparator);
        if absolute && !allow_absolute {
            self.error_span(
                code,
                self.previous().span,
                "a namespace declaration cannot begin with `\\`",
            );
            return None;
        }
        let first = self.expect(TokenKind::Identifier, code, message)?;
        let mut segments = vec![self.text(first.span).to_owned()];
        let mut span = if absolute {
            self.tokens[self.current.saturating_sub(2)]
                .span
                .join(first.span)
        } else {
            first.span
        };
        while self.consume(TokenKind::NamespaceSeparator) {
            let separator = self.previous().span;
            let Some(segment) = self.expect(
                TokenKind::Identifier,
                code,
                "expected a name segment after `\\`",
            ) else {
                self.error_span(code, separator, "a qualified name cannot end with `\\`");
                return None;
            };
            segments.push(self.text(segment.span).to_owned());
            span = span.join(segment.span);
        }
        Some(QualifiedName {
            absolute,
            segments,
            span,
        })
    }

    fn parse_name_ref(&mut self, code: &'static str, message: &'static str) -> Option<NameRef> {
        let name = self.parse_qualified_name(true, code, message)?;
        Some(NameRef {
            name: name.as_string(),
            span: name.span,
        })
    }

    fn current(&self) -> Token {
        self.tokens[self.current.min(self.tokens.len() - 1)]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current.saturating_sub(1)]
    }

    fn peek_kind(&self, distance: usize) -> TokenKind {
        self.tokens
            .get(self.current + distance)
            .map_or(TokenKind::Eof, |token| token.kind)
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    fn advance(&mut self) -> Token {
        let token = self.current();
        if token.kind != TokenKind::Eof {
            self.current += 1;
        }
        token
    }

    fn consume(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(
        &mut self,
        kind: TokenKind,
        code: &'static str,
        message: impl Into<String>,
    ) -> Option<Token> {
        if self.at(kind) {
            Some(self.advance())
        } else {
            self.error_current(code, message);
            None
        }
    }

    fn error_current(&mut self, code: &'static str, message: impl Into<String>) {
        self.error_span(code, self.current().span, message);
    }

    fn error_span(&mut self, code: &'static str, span: Span, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error("parsing", code, span, message));
    }

    fn synchronize(&mut self) {
        while !self.at(TokenKind::Eof) {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            if matches!(
                self.current().kind,
                TokenKind::Function
                    | TokenKind::Class
                    | TokenKind::Interface
                    | TokenKind::Trait
                    | TokenKind::Abstract
                    | TokenKind::Final
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::For
                    | TokenKind::Foreach
                    | TokenKind::Break
                    | TokenKind::Continue
                    | TokenKind::Return
                    | TokenKind::Echo
                    | TokenKind::RBrace
            ) {
                return;
            }
            self.advance();
        }
    }

    fn text(&self, span: Span) -> &str {
        &self.source[span.range()]
    }

    fn name_ref(&self, token: Token) -> NameRef {
        NameRef {
            name: self.text(token.span).to_owned(),
            span: token.span,
        }
    }
}

fn is_property_initializer(expression: &Expr) -> bool {
    match &expression.kind {
        ExprKind::Integer(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Null
        | ExprKind::String(_) => true,
        ExprKind::Vector(values) => values.iter().all(is_property_initializer),
        ExprKind::Map(entries) => entries.iter().all(|entry| {
            is_property_initializer(&entry.key) && is_property_initializer(&entry.value)
        }),
        ExprKind::Unary { operand, .. } => is_property_initializer(operand),
        ExprKind::Variable(_)
        | ExprKind::Name(_)
        | ExprKind::Binary { .. }
        | ExprKind::Call { .. }
        | ExprKind::Index { .. }
        | ExprKind::New { .. }
        | ExprKind::Property { .. }
        | ExprKind::MethodCall { .. }
        | ExprKind::StaticCall { .. }
        | ExprKind::ClassConstant { .. }
        | ExprKind::InstanceOf { .. }
        | ExprKind::Match { .. } => false,
    }
}

fn index_assignment_target(mut expression: Expr) -> Option<(String, Span, Vec<Expr>)> {
    let mut indices = Vec::new();
    loop {
        match expression.kind {
            ExprKind::Index { collection, index } => {
                indices.push(*index);
                expression = *collection;
            }
            ExprKind::Variable(name) => {
                indices.reverse();
                return Some((name, expression.span, indices));
            }
            _ => return None,
        }
    }
}

const fn binary_operator(kind: TokenKind) -> Option<(u8, BinaryOp)> {
    match kind {
        TokenKind::QuestionQuestion => Some((1, BinaryOp::Coalesce)),
        TokenKind::OrOr => Some((2, BinaryOp::Or)),
        TokenKind::AndAnd => Some((3, BinaryOp::And)),
        TokenKind::EqualEqual => Some((4, BinaryOp::Equal)),
        TokenKind::StrictEqual => Some((4, BinaryOp::StrictEqual)),
        TokenKind::StrictNotEqual | TokenKind::BangEqual => Some((4, BinaryOp::NotEqual)),
        TokenKind::Less => Some((5, BinaryOp::Less)),
        TokenKind::LessEqual => Some((5, BinaryOp::LessEqual)),
        TokenKind::Greater => Some((5, BinaryOp::Greater)),
        TokenKind::GreaterEqual => Some((5, BinaryOp::GreaterEqual)),
        TokenKind::Dot => Some((6, BinaryOp::Concatenate)),
        TokenKind::Plus => Some((7, BinaryOp::Add)),
        TokenKind::Minus => Some((7, BinaryOp::Subtract)),
        TokenKind::Star => Some((8, BinaryOp::Multiply)),
        TokenKind::Slash => Some((8, BinaryOp::Divide)),
        TokenKind::Percent => Some((8, BinaryOp::Remainder)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use thp_diagnostics::SourceFile;

    use super::parse;
    use crate::{BinaryOp, ExprKind, StmtKind};

    #[test]
    fn parses_functions_control_flow_and_precedence() {
        let source = SourceFile::new(
            "test.thp",
            r#"<?thp
function double(int $value): int {
    return $value * 2;
}
$answer: int = double(20 + 1);
if ($answer > 40) {
    echo "ok\n";
} else {
    echo "bad\n";
}
"#,
        );
        let output = parse(&source);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        assert_eq!(output.program.statements.len(), 3);
        let StmtKind::Assign { value, .. } = &output.program.statements[1].kind else {
            panic!("expected assignment");
        };
        let ExprKind::Call { arguments, .. } = &value.kind else {
            panic!("expected call");
        };
        assert!(matches!(
            arguments[0].value.kind,
            ExprKind::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn recovers_after_missing_expression() {
        let source = SourceFile::new("test.thp", "<?thp\n$x = ;\necho \"after\";");
        let output = parse(&source);
        assert!(!output.diagnostics.is_empty());
        assert!(
            output
                .program
                .statements
                .iter()
                .any(|statement| matches!(statement.kind, StmtKind::Echo(_)))
        );
    }

    #[test]
    fn rejects_missing_open_tag() {
        let source = SourceFile::new("test.thp", "echo \"no\";");
        let output = parse(&source);
        assert_eq!(output.diagnostics[0].code, "P0001");
    }

    #[test]
    fn rejects_comma_separated_echo_operands() {
        let source = SourceFile::new("test.thp", "<?thp\necho $choices[0], \"\\n\";");
        let output = parse(&source);
        assert_eq!(output.diagnostics[0].code, "P0302");
        assert_eq!(output.diagnostics[0].labels[0].span.start, 22);
    }

    #[test]
    fn parses_essential_control_flow_and_argument_forms() {
        let source = SourceFile::new(
            "test.thp",
            r"<?thp
function collect(int $base = 1, int ...$rest): int { return $base; }
$items: vector<int> = [1];
for ($i: int = 0; $i < 1; $i = $i + 1) {
    foreach ($items as $key => $value) {
        if ($key === 0) { continue; }
        break;
    }
}
$items[0] = match ($items[0]) { 1, 2 => collect(base: 3), default => 0 };
",
        );
        let output = parse(&source);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let StmtKind::Function(function) = &output.program.statements[0].kind else {
            panic!("expected function");
        };
        assert!(function.parameters[0].default.is_some());
        assert!(function.parameters[1].variadic);
        assert!(matches!(
            output.program.statements[2].kind,
            StmtKind::For { .. }
        ));
        let StmtKind::SetIndex { value, indices, .. } = &output.program.statements[3].kind else {
            panic!("expected collection assignment");
        };
        assert_eq!(indices.len(), 1);
        assert!(matches!(value.kind, ExprKind::Match { .. }));
    }

    #[test]
    fn parses_nominal_declarations_traits_scoped_calls_and_finally() {
        let source = SourceFile::new(
            "test.thp",
            r#"<?thp
interface ParentContract {
    public function render(string $value): string;
    public static function kind(): string;
}
interface ChildContract extends ParentContract {}
interface Cacheable {
    public function clear(): void;
}
trait Rendering {
    abstract protected function prefix(): string;
    public function render(string $value): string {
        return self::prefix() . static::kind() . parent::kind() . $value;
    }
}
abstract class Base implements ChildContract {
    abstract protected function prefix(): string;
    public static function kind(): string { return "base"; }
}
final class Page extends Base implements Cacheable {
    use Rendering;
    protected function prefix(): string { return "page"; }
    public function clear(): void {}
}
try {
    echo Page::kind();
} finally {
    echo "done";
}
"#,
        );
        let output = parse(&source);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        assert!(matches!(
            output.program.statements[0].kind,
            StmtKind::Interface(_)
        ));
        assert!(matches!(
            output.program.statements[3].kind,
            StmtKind::Trait(_)
        ));
        assert!(matches!(
            output.program.statements[4].kind,
            StmtKind::Class(_)
        ));
        let StmtKind::Try {
            catches, finally, ..
        } = &output.program.statements[6].kind
        else {
            panic!("expected try statement");
        };
        assert!(catches.is_empty());
        assert!(finally.is_some());
    }

    #[test]
    fn diagnoses_invalid_interface_bodies_and_multiple_parents() {
        let source = SourceFile::new(
            "test.thp",
            r"<?thp
interface Multiple extends First, Second {}
interface WithBody {
    public function render(): void {}
}
",
        );
        let output = parse(&source);
        let codes = output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"P0162"));
        assert!(codes.contains(&"P0165"));
    }

    #[test]
    fn parses_namespace_imports_and_absolute_references() {
        let source = SourceFile::new(
            "test.thp",
            r"<?thp
namespace App\Service;
use Vendor\Package\Client;
use Vendor\Package\Other as PackageClient;
use function Vendor\Package\makeClient;
function build(\Vendor\Contracts\Client $client): Client {
    return makeClient();
}
",
        );
        let output = parse(&source);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let namespace = output.program.namespace.as_ref().unwrap();
        assert_eq!(namespace.name.segments, ["App", "Service"]);
        assert_eq!(output.program.imports.len(), 3);
        assert_eq!(
            output.program.imports[1].target.segments,
            ["Vendor", "Package", "Other"]
        );
        let StmtKind::Function(function) = &output.program.statements[0].kind else {
            panic!("expected function");
        };
        let crate::TypeSyntaxKind::Named { name, .. } = &function.parameters[0].ty.kind else {
            panic!("expected named type");
        };
        assert_eq!(name, "\\Vendor\\Contracts\\Client");
    }

    #[test]
    fn rejects_late_multiple_and_unsupported_import_forms() {
        for (source, code) in [
            ("<?thp\nclass A {}\nuse Vendor\\A;\n", "P0004"),
            ("<?thp\nnamespace App;\nnamespace Other;\n", "P0003"),
            ("<?thp\nuse const Vendor\\VALUE;\n", "P0010"),
            ("<?thp\nuse Vendor\\A;\nuse Vendor\\B as A;\n", "P0013"),
        ] {
            let output = parse(&SourceFile::new("test.thp", source));
            assert!(
                output
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == code),
                "{source:?}: {:?}",
                output.diagnostics
            );
        }
    }
}
