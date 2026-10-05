---
kind: class
id: std.spl.LogicException
title: LogicException
summary: Reports a problem detectable from program logic.
name: LogicException
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

`LogicException` reports a problem detectable from program logic.

## Role

`LogicException` reports a problem detectable from program logic. No THP standard-library operation is currently specified to throw it.

## Construction

Construction and diagnostic access are inherited from `Exception`.

## Example

```thp
throw new LogicException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `LogicException`](https://www.php.net/manual/en/class.logicexception.php)
