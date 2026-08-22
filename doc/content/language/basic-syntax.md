---
kind: guide
id: guide.languageBasicSyntax
title: Basic syntax
summary: Describes THP source files, tags, statements, and comments.
nav:
  section: language
  order: 20
status: experimental
availability: partial
notice: >-
  The current bytecode interpreter executes the core tags, statements, blocks,
  and comments on this page. Class and exception blocks remain proposals; see
  Implementation status for the exact executable subset.
---

THP source uses familiar PHP-shaped tokens with explicit statement terminators
and brace-delimited blocks.

## Source files

A THP program starts with `<?thp`. Source following the opening tag is parsed as
THP code.

```thp
<?thp

echo "Hello, THP!\n";
```

THP is designed to compile and run in a standalone runtime. A `.thp` file is
not a PHP script and does not target PHP's engine.

## Statements and blocks

Simple statements end with `;`. Braces group statements for functions, classes,
branches, loops, and exception handlers.

```thp
$ready = true;

if ($ready) {
    echo "ready\n";
}
```

## Output

`echo` accepts exactly one expression whose static type is `string`, `int`,
`float`, or `bool`. Comma-separated operand lists are a parser error. Output
uses the same conversion as the string concatenation operator:

| Type     | Output conversion                                                 |
| -------- | ----------------------------------------------------------------- |
| `string` | The original bytes.                                               |
| `int`    | Base-10 digits with a leading `-` when negative.                  |
| `float`  | The canonical, locale-independent representation described below. |
| `bool`   | The lowercase literal `true` or `false`.                          |

A finite `float` uses the shortest decimal representation that parses back to
the same double-precision value. An integer-valued representation retains a
floating-point marker, so `1.0` produces `1.0` and negative zero produces
`-0.0`. The non-finite values produce `NAN`, `INF`, and `-INF`. There is no
process-wide or request-wide output precision setting; calculated binary
floating-point differences therefore remain visible.

`null`, nullable types, `mixed`, collections, and objects are rejected
statically. Handle a nullable value explicitly by narrowing it or providing a
fallback with `??`. Use `var_dump()` when the type and value need to be
inspected explicitly.

```thp
echo 3;
echo "items: " . 3 . "\n";
echo $name ?? "anonymous";
var_dump(true, null);
```

## Comments

THP recognizes PHP-shaped line and block comments.

```thp
// A line comment.
/* A block comment. */
/*
 Multi-line block comment.
*/
```

Comments have no runtime effect.

## See also

- [Expressions](thp:guide.languageExpressions)
- [Control structures](thp:guide.languageControlStructures)
- [Implementation status](thp:guide.implementationStatus)
- [THP for PHP developers](thp:guide.phpDevelopers)
