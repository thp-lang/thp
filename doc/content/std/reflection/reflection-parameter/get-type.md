---
kind: method
id: std.reflection.ReflectionParameter::getType
title: ReflectionParameter::getType
summary: Returns the declared parameter type with generic substitution.
name: getType
order: 76
typeParameters: []
parameters: []
returns:
  type: ReflectionType
  description: Returns the declared parameter type with generic substitution.
errors: []
related:
  - std.reflection.ReflectionParameter
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionParameter
visibility: public
modifiers: []
---

Returns the declared parameter type with generic substitution.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionParameter`](thp:std.reflection.ReflectionParameter)
