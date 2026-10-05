---
kind: class
id: std.spl.BadMethodCallException
title: BadMethodCallException
summary: Reports an invalid method call.
name: BadMethodCallException
module: exceptions
typeParameters: []
parent:
  id: std.spl.BadFunctionCallException
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This native exception class is constructible and catchable.
version: "0.1"
---

`BadMethodCallException` reports an invalid method call.

## Role

`BadMethodCallException` reports an invalid method call. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `BadFunctionCallException`.

## Example

```thp
throw new BadMethodCallException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `BadMethodCallException`](https://www.php.net/manual/en/class.badmethodcallexception.php)
