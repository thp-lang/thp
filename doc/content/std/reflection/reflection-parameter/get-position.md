---
kind: method
id: std.reflection.ReflectionParameter::getPosition
title: ReflectionParameter::getPosition
summary: Returns the zero-based parameter position.
name: getPosition
order: 75
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the zero-based parameter position.
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

Returns the zero-based parameter position.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionParameter`](thp:std.reflection.ReflectionParameter)
