---
kind: guide
id: guide.lexingInternals
title: Lexing
summary: Learn how source bytes become spanned tokens without losing diagnostic locations.
nav:
  section: internals
  order: 30
status: experimental
availability: implemented
notice: >-
  The lexer documents the executable token set; unsupported PHP tokens are not
  implicitly accepted by THP.
---

The lexer in `thp-syntax` scans one `SourceFile` and returns tokens plus any
lexical diagnostics. Tokens identify syntax categories; the source map remains
the owner of the original spelling.

```thp
<?thp
echo "Hello";
```

Inspect it with:

```console
thp inspect --emit=tokens hello.thp
```

The current output is a Rust debugging representation. This excerpt is
abbreviated, but the token names and byte spans come from the command:

```text
Token { kind: OpenTag, span: Span { start: 0, end: 5 } }
Token { kind: Echo, span: Span { start: 6, end: 10 } }
Token { kind: String, span: Span { start: 11, end: 18 } }
Token { kind: Semicolon, span: Span { start: 18, end: 19 } }
Token { kind: Eof, span: Span { start: 20, end: 20 } }
```

Spans use byte offsets into the UTF-8 source. Keeping them on every token lets
the parser and later semantic phases report the original expression rather
than an internal node location.

## What the lexer decides

Lexing recognizes the required `<?thp` tag, identifiers and variables,
keywords, numeric and string literals, punctuation, operators, whitespace, and
comments. It decodes string literal contents into bytes while retaining the
literal span. It also appends an explicit end-of-file token.

The lexer does not decide precedence, declaration structure, whether a name
exists, or whether an expression has the expected type. Those belong to later
phases. When a byte sequence cannot form a valid token, the lexer records a
diagnostic and continues far enough for the parser to report useful neighboring
problems.

## Design choices compared with PHP

PHP began as a template language, so its lexer supports transitions between
inline document text and `<?php ... ?>` code regions. THP chooses code-only
source files: one required `<?thp` opening tag switches the rest of the file to
THP syntax, with no closing-tag or inline-HTML mode. This removes a lexer state
that a standalone application compiler does not need.

THP also requires UTF-8 source and currently limits identifiers to ASCII, which
makes byte spans and diagnostics deterministic across hosts. This does not make
`string` a Unicode text type: like PHP strings, THP runtime strings remain
arbitrary bytes, and text-aware APIs must validate an encoding explicitly.

The resulting token stream is consumed by the [parser](thp:guide.parsingAst).
