---
kind: class
id: std.reflection.ReflectionException
title: ReflectionException
summary: Reports an invalid reflective operation.
name: ReflectionException
module: reflection
typeParameters: []
parent:
  id: std.baseTypes.Exception
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This native throwable is implemented and may be extended.
version: "0.1"
---

`ReflectionException` is thrown for missing reflected declarations, incompatible receivers, and missing or non-public reflected constructors. Dynamic type and count failures use `TypeError` and `ArgumentCountError`; exceptions thrown by reflected user code propagate unchanged.

## See also

- [Reflection](thp:std.reflection)
