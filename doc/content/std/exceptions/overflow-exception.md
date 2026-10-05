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
availability: implemented
notice: This native exception class is constructible and catchable.
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

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `OverflowException`](https://www.php.net/manual/en/class.overflowexception.php)
