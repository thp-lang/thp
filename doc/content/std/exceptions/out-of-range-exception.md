---
kind: class
id: std.spl.OutOfRangeException
title: OutOfRangeException
summary: Reports an index or value outside a logically valid range.
name: OutOfRangeException
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

`OutOfRangeException` reports an index or value outside a logically valid range.

## Role

`OutOfRangeException` reports an index or value outside a logically valid range. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `LogicException`.

## Example

```thp
throw new OutOfRangeException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `OutOfRangeException`](https://www.php.net/manual/en/class.outofrangeexception.php)
