---
kind: method
id: std.reflection.ReflectionProperty::getDeclaringClass
title: ReflectionProperty::getDeclaringClass
summary: Returns the consuming or declaring class.
name: getDeclaringClass
order: 63
typeParameters: []
parameters: []
returns:
  type: ReflectionClass
  description: Returns the consuming or declaring class.
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

Returns the consuming or declaring class.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
