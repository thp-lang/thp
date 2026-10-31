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
availability: implemented
notice: This native exception class is constructible and catchable.
version: "0.1"
---

`OutOfBoundsException` reports access beyond available bounds.

## Role

`OutOfBoundsException` reports access beyond available bounds. `Option::get()` throws it for an absent option.

## Construction

Construction and diagnostic access are inherited from `RuntimeException`.

## Example

```thp
throw new OutOfBoundsException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `OutOfBoundsException`](https://www.php.net/manual/en/class.outofboundsexception.php)
