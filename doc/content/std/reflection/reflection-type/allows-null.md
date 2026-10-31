---
kind: method
id: std.reflection.ReflectionType::allowsNull
title: ReflectionType::allowsNull
summary: Reports whether the reflected type accepts null.
name: allowsNull
order: 1
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the reflected type accepts null.
errors: []
related:
  - std.reflection.ReflectionType
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionType
visibility: public
modifiers: []
---

Reports whether the reflected type accepts null.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionType`](thp:std.reflection.ReflectionType)
