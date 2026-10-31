---
kind: class
id: std.spl.UnexpectedValueException
title: UnexpectedValueException
summary: Reports a value that does not satisfy an operation’s expectation.
name: UnexpectedValueException
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

`UnexpectedValueException` reports a value that does not satisfy an operation’s expectation.

## Role

`UnexpectedValueException` reports a value that does not satisfy an operation’s expectation. It is thrown for negative `Countable::count()` results and malformed input to `unserialize()`.

## Construction

Construction and diagnostic access are inherited from `RuntimeException`.

## Example

```thp
throw new UnexpectedValueException("operation failed");
```

The class and inheritance are executable; failure sites are documented by each API.

## See also

- [SPL exceptions](thp:std.exceptions)
- [PHP `UnexpectedValueException`](https://www.php.net/manual/en/class.unexpectedvalueexception.php)
