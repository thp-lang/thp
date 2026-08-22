---
kind: class
id: std.spl.BadFunctionCallException
title: BadFunctionCallException
summary: Reports an invalid function call.
name: BadFunctionCallException
module: exceptions
typeParameters: []
parent:
  id: std.spl.LogicException
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

`BadFunctionCallException` reports an invalid function call.

## Role

`BadFunctionCallException` reports an invalid function call. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `LogicException`.

## Example

```thp
throw new BadFunctionCallException("operation failed");
```

The example demonstrates the proposed inheritance name only; concrete APIs do not yet promise this failure type.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `BadFunctionCallException`](https://www.php.net/manual/en/class.badfunctioncallexception.php)
