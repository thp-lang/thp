---
kind: class
id: std.baseTypes.ArgumentCountError
title: ArgumentCountError
summary: Reports a dynamic argument count mismatch.
name: ArgumentCountError
module: base-types
typeParameters: []
parent:
  id: std.baseTypes.TypeError
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: The reference VM throws this native error for implemented dynamic argument binding.
version: "0.1"
---

`ArgumentCountError` extends [`TypeError`](thp:std.baseTypes.TypeError) and reports missing or excess runtime arguments.

## See also

- [`TypeError`](thp:std.baseTypes.TypeError)
- [Errors](thp:guide.languageErrors)
