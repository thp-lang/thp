---
kind: method
id: std.reflection.ReflectionNamedType::getName
title: ReflectionNamedType::getName
summary: Returns the scalar, collection, nominal, or type-parameter name.
name: getName
order: 5
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the scalar, collection, nominal, or type-parameter name.
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

Returns the scalar, collection, nominal, or type-parameter name.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionNamedType`](thp:std.reflection.ReflectionNamedType)
