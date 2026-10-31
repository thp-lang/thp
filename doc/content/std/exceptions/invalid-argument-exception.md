---
kind: class
id: std.spl.InvalidArgumentException
title: InvalidArgumentException
summary: Reports an argument that violates a function contract.
name: InvalidArgumentException
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

`InvalidArgumentException` reports an argument that violates a function contract.

## Role

`InvalidArgumentException` reports an argument that violates a function contract. `serialize()` and `TraceLine::__construct()` use it for unsupported values.

## Construction

Construction and diagnostic access are inherited from `LogicException`.

## Example

```thp
throw new InvalidArgumentException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `InvalidArgumentException`](https://www.php.net/manual/en/class.invalidargumentexception.php)
