---
kind: method
id: std.reflection.ReflectionFunctionAbstract::isInternal
title: ReflectionFunctionAbstract::isInternal
summary: Reports whether the callable is runtime-owned.
name: isInternal
order: 47
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the callable is runtime-owned.
errors: []
related:
  - std.reflection.ReflectionFunctionAbstract
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionFunctionAbstract
visibility: public
modifiers: []
---

Reports whether the callable is runtime-owned.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract)
