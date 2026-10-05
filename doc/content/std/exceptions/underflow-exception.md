---
kind: class
id: std.spl.UnderflowException
title: UnderflowException
summary: Reports removal from an empty container.
name: UnderflowException
module: exceptions
typeParameters: []
parent:
  id: std.spl.RuntimeException
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This native exception class is constructible and catchable.
version: "0.1"
---

`UnderflowException` reports removal from an empty container.

## Role

`UnderflowException` reports removal from an empty container. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `RuntimeException`.

## Example

```thp
throw new UnderflowException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `UnderflowException`](https://www.php.net/manual/en/class.underflowexception.php)
