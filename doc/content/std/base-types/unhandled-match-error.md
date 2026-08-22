---
kind: class
id: std.baseTypes.UnhandledMatchError
title: UnhandledMatchError
summary: Reports a match expression for which no arm was selected.
name: UnhandledMatchError
module: base-types
typeParameters: []
parent:
  id: std.baseTypes.Error
interfaces: []
constants: []
properties: []
status: experimental
availability: partial
notice: >-
  The compiler and reference VM throw this native error for an unmatched
  match expression. Exact message wording remains experimental.
version: "0.1"
---

`UnhandledMatchError` is thrown when a `match` expression reaches the end of
its arms without a matching condition or a `default` arm.

## Behavior

The subject is evaluated once. The runtime message includes a bounded,
deterministic description of that subject: scalar values are shown directly,
long strings are escaped and truncated, and compound values are described by
type. Constructing the message does not invoke application code.

The error is catchable as `UnhandledMatchError` or
[`Error`](thp:std.baseTypes.Error). Adding a `default` arm makes the expression
exhaustive at runtime.

## Example

```thp
try {
    $label = match ($status) {
        200 => "ok",
        404 => "missing",
    };
} catch (UnhandledMatchError $error) {
    echo $error->getMessage();
}
```

## See also

- [`Error`](thp:std.baseTypes.Error)
- [Control structures](thp:guide.languageControlStructures)
- [Errors](thp:guide.languageErrors)
