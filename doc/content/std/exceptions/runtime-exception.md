---
kind: class
id: std.spl.RuntimeException
title: RuntimeException
summary: Reports a failure detected only while the program runs.
name: RuntimeException
module: exceptions
typeParameters: []
parent:
  id: std.baseTypes.Exception
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This native exception class is constructible and catchable.
version: "0.1"
---

`RuntimeException` reports a failure detected only while the program runs.

## Role

`RuntimeException` reports a failure detected only while the program runs. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `Exception`.

## Example

```thp
throw new RuntimeException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `RuntimeException`](https://www.php.net/manual/en/class.runtimeexception.php)
