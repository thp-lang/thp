---
kind: class
id: std.baseTypes.TypeError
title: TypeError
summary: Reports a dynamic argument type mismatch.
name: TypeError
module: base-types
typeParameters: []
parent:
  id: std.baseTypes.Error
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: The reference VM throws this native error for implemented dynamic type checks.
version: "0.1"
---

`TypeError` extends [`Error`](thp:std.baseTypes.Error) and reports a runtime value that does not satisfy an API's required type.

## See also

- [`ArgumentCountError`](thp:std.baseTypes.ArgumentCountError)
- [Errors](thp:guide.languageErrors)
