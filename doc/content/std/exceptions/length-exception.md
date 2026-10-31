---
kind: class
id: std.spl.LengthException
title: LengthException
summary: Reports an invalid length.
name: LengthException
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

`LengthException` reports an invalid length.

## Role

`LengthException` reports an invalid length. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `LogicException`.

## Example

```thp
throw new LengthException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `LengthException`](https://www.php.net/manual/en/class.lengthexception.php)
