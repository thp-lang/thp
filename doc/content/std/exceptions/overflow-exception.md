---
kind: class
id: std.spl.OverflowException
title: OverflowException
summary: Reports insertion into a full container.
name: OverflowException
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

`OverflowException` reports insertion into a full container.

## Role

`OverflowException` reports insertion into a full container. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `RuntimeException`.

## Example

```thp
throw new OverflowException("operation failed");
```

The example demonstrates the proposed inheritance name only; concrete APIs do not yet promise this failure type.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `OverflowException`](https://www.php.net/manual/en/class.overflowexception.php)
