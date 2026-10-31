---
kind: method
id: std.reflection.ReflectionNamedType::isBuiltin
title: ReflectionNamedType::isBuiltin
summary: Reports whether the name denotes a built-in scalar or collection type.
name: isBuiltin
order: 6
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the name denotes a built-in scalar or collection type.
errors: []
related:
  - std.reflection.ReflectionNamedType
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionNamedType
visibility: public
modifiers: []
---

Reports whether the name denotes a built-in scalar or collection type.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionNamedType`](thp:std.reflection.ReflectionNamedType)
