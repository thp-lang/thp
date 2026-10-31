---
kind: method
id: std.reflection.ReflectionClass::getDeclaredProperties
title: ReflectionClass::getDeclaredProperties
summary: Returns properties declared by this nominal in deterministic order.
name: getDeclaredProperties
order: 32
typeParameters: []
parameters: []
returns:
  type: vector<ReflectionProperty>
  description: Returns properties declared by this nominal in deterministic order.
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

Returns properties declared by this nominal in deterministic order.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
