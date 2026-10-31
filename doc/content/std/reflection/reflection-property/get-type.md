---
kind: method
id: std.reflection.ReflectionProperty::getType
title: ReflectionProperty::getType
summary: Returns the declared property type with concrete generic substitution.
name: getType
order: 65
typeParameters: []
parameters: []
returns:
  type: ReflectionType
  description: Returns the declared property type with concrete generic substitution.
errors: []
related:
  - std.reflection.ReflectionProperty
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionProperty
visibility: public
modifiers: []
---

Returns the declared property type with concrete generic substitution.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
