---
kind: interface
id: std.baseTypes.Throwable
title: Throwable
summary: Defines the common information exposed by objects that can be thrown.
name: Throwable
module: base-types
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: partial
notice: >-
  The executable object model implements the message, code, previous, and
  suppressed-failure slice. Origin and trace inspection and string conversion
  remain proposed.
version: "0.1"
---

`Throwable` is the sealed base interface for objects accepted by `throw` and
produced by error handling.

## Contract

The executable interface provides an error message, code, optional previous
throwable, and suppressed cleanup failures. The broader reference contract also
reserves origin, stack trace, and string representation. Ordinary classes
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
