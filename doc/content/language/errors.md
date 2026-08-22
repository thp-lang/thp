---
kind: guide
id: guide.languageErrors
title: Errors
summary: Describes THP compile diagnostics and runtime language failures.
nav:
  section: language
  order: 130
status: experimental
availability: partial
notice: >-
  Structured compile diagnostics and the catchable unmatched-match error
  execute. Additional public runtime-error classes and reporting configuration
  remain unsettled.
---

THP reports failures at the earliest boundary that can establish them.

## Compile diagnostics

Lexing, parsing, name resolution, typing, lowering, and bytecode verification
may reject a program before it runs. A compile diagnostic includes its phase,
stable diagnostic code, primary source span, relevant related spans, and notes
such as the valid parameter names where that information is available.

Control-flow and call diagnostics include invalid loop transfers, unsupported
numeric levels and call-site unpacking, invalid `foreach` sources or bindings,
invalid `match` arms, invalid collection paths, default-value violations, and
unknown, duplicate, missing, or misplaced arguments.

## Runtime failures

Operations that depend on runtime data can fail during execution. Examples
include invalid collection bounds, arithmetic failures, stream failures, and
unmatched `match` expressions.

An unmatched `match` throws
[`UnhandledMatchError`](thp:std.baseTypes.UnhandledMatchError), which derives
from [`Error`](thp:std.baseTypes.Error). Typed `try`/`catch` may handle either
type. If it escapes the entry function, the runtime reports the class name,
message, source span, and accumulated call trace rather than printing or
terminating from a compiler phase.

The proposed public classes are listed under
Predefined exceptions. Their names and inheritance
remain experimental rather than a compatibility promise.

## See also

- [Exceptions](thp:guide.languageExceptions)
- Predefined exceptions
- [Types](thp:guide.languageTypes)
