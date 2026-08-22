---
kind: class
id: std.spl.UnexpectedValueException
title: UnexpectedValueException
summary: Reports a value that does not satisfy an operation’s expectation.
name: UnexpectedValueException
module: exceptions
typeParameters: []
parent:
  id: std.spl.RuntimeException
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

`UnexpectedValueException` reports a value that does not satisfy an operation’s expectation.

## Role

`UnexpectedValueException` reports a value that does not satisfy an operation’s expectation. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `RuntimeException`.

## Example

```thp
throw new UnexpectedValueException("operation failed");
```

The example demonstrates the proposed inheritance name only; concrete APIs do not yet promise this failure type.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `UnexpectedValueException`](https://www.php.net/manual/en/class.unexpectedvalueexception.php)
