---
kind: class
id: std.spl.RuntimeException
title: RuntimeException
summary: Reports a failure detected only while the program runs.
name: RuntimeException
module: exceptions
typeParameters: []
parent:
  id: std.baseTypes.Exception
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired exception class contract is proposed and is not implemented in
  this repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`RuntimeException` reports a failure detected only while the program runs.

## Role

`RuntimeException` reports a failure detected only while the program runs. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `Exception`.

## Example

```thp
throw new RuntimeException("operation failed");
```

The example demonstrates the proposed inheritance name only; concrete APIs do not yet promise this failure type.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `RuntimeException`](https://www.php.net/manual/en/class.runtimeexception.php)
