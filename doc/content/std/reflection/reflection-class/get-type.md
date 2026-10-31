---
kind: method
id: std.reflection.ReflectionClass::getType
title: ReflectionClass::getType
summary: Returns the concrete nominal type when generic arguments are known.
name: getType
order: 15
typeParameters: []
parameters: []
returns:
  type: ?ReflectionNamedType
  description: Returns the concrete nominal type when generic arguments are known.
errors: []
related:
  - std.reflection.ReflectionClass
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionClass
visibility: public
modifiers: []
---

Returns the concrete nominal type when generic arguments are known.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
