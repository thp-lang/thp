---
kind: method
id: std.reflection.ReflectionNamedType::getTypeArguments
title: ReflectionNamedType::getTypeArguments
summary: Returns concrete generic arguments in declaration order.
name: getTypeArguments
order: 8
typeParameters: []
parameters: []
returns:
  type: vector<ReflectionType>
  description: Returns concrete generic arguments in declaration order.
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

Returns concrete generic arguments in declaration order.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionNamedType`](thp:std.reflection.ReflectionNamedType)
