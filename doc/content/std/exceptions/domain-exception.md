---
kind: class
id: std.spl.DomainException
title: DomainException
summary: Reports a value outside a defined semantic domain.
name: DomainException
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

`DomainException` reports a value outside a defined semantic domain.

## Role

`DomainException` reports a value outside a defined semantic domain. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `LogicException`.

## Example

```thp
throw new DomainException("operation failed");
```

The example demonstrates the proposed inheritance name only; concrete APIs do not yet promise this failure type.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `DomainException`](https://www.php.net/manual/en/class.domainexception.php)
