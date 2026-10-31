---
kind: class
id: std.spl.RangeException
title: RangeException
summary: Reports an invalid range discovered at runtime.
name: RangeException
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

`RangeException` reports an invalid range discovered at runtime.

## Role

`RangeException` reports an invalid range discovered at runtime. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `RuntimeException`.

## Example

```thp
throw new RangeException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `RangeException`](https://www.php.net/manual/en/class.rangeexception.php)
