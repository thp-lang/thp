---
kind: method
id: std.reflection.ReflectionNamedType::isTypeParameter
title: ReflectionNamedType::isTypeParameter
summary: Reports whether the descriptor denotes a declaration type parameter.
name: isTypeParameter
order: 7
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the descriptor denotes a declaration type parameter.
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

Reports whether the descriptor denotes a declaration type parameter.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionNamedType`](thp:std.reflection.ReflectionNamedType)
