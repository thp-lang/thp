---
kind: class
id: std.spl.OutOfBoundsException
title: OutOfBoundsException
summary: Reports access beyond available bounds.
name: OutOfBoundsException
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

`OutOfBoundsException` reports access beyond available bounds.

## Role

`OutOfBoundsException` reports access beyond available bounds. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `RuntimeException`.

## Example

```thp
throw new OutOfBoundsException("operation failed");
```

The example demonstrates the proposed inheritance name only; concrete APIs do not yet promise this failure type.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `OutOfBoundsException`](https://www.php.net/manual/en/class.outofboundsexception.php)
