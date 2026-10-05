---
kind: interface
id: std.baseTypes.Throwable
title: Throwable
summary: Defines the common information exposed by objects that can be thrown.
name: Throwable
module: base-types
typeParameters: []
interfaces:
  - id: std.baseTypes.Stringable
constants: []
properties: []
status: experimental
availability: implemented
notice: The compiler and reference VM implement this experimental contract.
version: "0.1"
---

`Throwable` is the sealed base interface for objects accepted by `throw` and
produced by error handling.

## Contract

The executable interface provides an error message, code, optional previous
throwable, suppressed cleanup failures, creation file and line, a structured
stack trace, and string representations. Trace frames are captured at
construction, ordered from the current call toward its callers. Receiver
objects and arguments are omitted from captured frames. Ordinary classes
cannot implement `Throwable` directly; throwable types derive from the
language's exception hierarchy.

## Example

```thp
function reportFailure(Throwable $failure): void {
    echo $failure->getMessage() . ":" . $failure->getCode();
    var_dump(count($failure->getSuppressed()));
}
```

## See also

- [`Exception`](thp:std.baseTypes.Exception)
- [`Error`](thp:std.baseTypes.Error)
- [`TraceLine`](thp:std.baseTypes.TraceLine)
