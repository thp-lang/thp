---
kind: class
id: std.baseTypes.Error
title: Error
summary: Base class for engine-detected language errors.
name: Error
module: base-types
typeParameters: []
interfaces:
  - id: std.baseTypes.Throwable
constants: []
properties: []
status: experimental
availability: partial
notice: >-
  The compiler and reference VM implement Error as a native throwable root.
  Its wider set of diagnostic accessors and constructor metadata remain
  experimental.
version: "0.1"
---

`Error` is the native root for catchable failures detected by the language
engine rather than explicitly modeled as application exceptions. It is
separate from [`Exception`](thp:std.baseTypes.Exception).

## Behavior

An `Error` carries a string message and supports the native error diagnostic
methods. A `catch (Exception $error)` does not select an `Error`; catch
`Error` or the concrete subclass.

The reference VM reports an uncaught error with its concrete class name,
message, source span, and call trace.

## Example

```thp
try {
    echo match (404) {
        200 => "ok",
    };
} catch (Error $error) {
    echo $error->getMessage();
}
```

## See also

- [`Exception`](thp:std.baseTypes.Exception)
- [`UnhandledMatchError`](thp:std.baseTypes.UnhandledMatchError)
