---
kind: class
id: std.spl.BadMethodCallException
title: BadMethodCallException
summary: Reports an invalid method call.
name: BadMethodCallException
module: exceptions
typeParameters: []
parent:
  id: std.spl.BadFunctionCallException
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

`BadMethodCallException` reports an invalid method call.

## Role

`BadMethodCallException` reports an invalid method call. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `BadFunctionCallException`.

## Example

```thp
throw new BadMethodCallException("operation failed");
```

The example demonstrates the proposed inheritance name only; concrete APIs do not yet promise this failure type.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `BadMethodCallException`](https://www.php.net/manual/en/class.badmethodcallexception.php)
