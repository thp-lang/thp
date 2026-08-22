---
kind: guide
id: guide.parsingAst
title: Parsing and AST
summary: See how the recovering parser builds a spanned syntax tree and preserves useful errors.
nav:
  section: internals
  order: 40
status: experimental
availability: implemented
notice: >-
  AST shapes are compiler debugging interfaces rather than a stable public
  serialization format.
---

The parser consumes the complete token stream and builds an abstract syntax
tree (AST). The tree describes what the source says without deciding whether
names resolve or types agree.

```thp
<?thp
function greet(string $name): string {
    return "Hello, " . $name;
}
```

`thp inspect --emit=ast greet.thp` prints the current tree. A shortened excerpt
looks like this:

```text
Stmt {
  kind: Function(FunctionDecl {
    name: "greet",
    parameters: [Parameter { name: "name", ty: Named("string"), ... }],
    return_type: Named("string"),
    body: [Stmt { kind: Return(Some(Expr {
      kind: Binary { op: Concatenate, ... },
      span: Span { start: 56, end: 73 }
    })) }]
  }),
  span: Span { start: 6, end: 76 }
}
```

The printed form is abbreviated here: the actual dump includes the span of
each name, type, argument, statement, and expression.

## Structure and recovery

The AST represents namespaces and imports, declarations, statements, type
syntax, and precedence-shaped expressions. A qualified name is still syntax at
this point. A type such as `vector<User>` records its written name and type
arguments rather than a resolved semantic type.

The parser uses precedence parsing for expressions and targeted routines for
declarations and statements. When it encounters malformed input, it emits a
structured diagnostic and synchronizes at a safe token such as a semicolon or
block boundary. That recovery lets one compile report multiple independent
syntax errors.

Lexer diagnostics are passed into parsing and returned with parser diagnostics.
If any frontend diagnostic remains, the compiler retains tokens and AST for
inspection but does not construct HIR.

## Design choices compared with PHP

PHP's grammar carries decades of compatibility features, including template
code regions, multiple namespace forms, dynamic expressions, and alternative
spellings for several control structures. THP deliberately accepts a smaller,
regular grammar: a file has at most one semicolon-style namespace, imports stay
at the front, and declarations and blocks follow the forms the static pipeline
can analyze.

THP's parser is also designed to recover at statement and block boundaries and
return several spanned syntax diagnostics in one compilation. Ordinary PHP
compilation stops the affected script at a parse error; multi-error recovery is
not a language-facing contract. THP pays for a more explicit recovering AST so
editors and build tools can present more problems without rerunning the parser
after every fix.

For a project, parsed AST units next enter
[module and name resolution](thp:guide.modulesResolution). A single source goes
directly to semantic analysis after its syntax succeeds.
