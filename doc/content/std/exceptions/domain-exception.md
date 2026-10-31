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
availability: implemented
notice: This native exception class is constructible and catchable.
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

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `DomainException`](https://www.php.net/manual/en/class.domainexception.php)
