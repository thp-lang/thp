---
kind: method
id: std.reflection.ReflectionClass::getProperties
title: ReflectionClass::getProperties
summary: Returns effective properties with declared properties first.
name: getProperties
order: 34
typeParameters: []
parameters: []
returns:
  type: vector<ReflectionProperty>
  description: Returns effective properties with declared properties first.
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

Returns effective properties with declared properties first.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
