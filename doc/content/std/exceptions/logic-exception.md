---
kind: class
id: std.spl.LogicException
title: LogicException
summary: Reports a problem detectable from program logic.
name: LogicException
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

`LogicException` reports a problem detectable from program logic.

## Role

`LogicException` reports a problem detectable from program logic. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `Exception`.

## Example

```thp
throw new LogicException("operation failed");
```

The example demonstrates the proposed inheritance name only; concrete APIs do not yet promise this failure type.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `LogicException`](https://www.php.net/manual/en/class.logicexception.php)
