---
kind: method
id: std.reflection.ReflectionClass::getDeclaredMethods
title: ReflectionClass::getDeclaredMethods
summary: Returns methods declared by this nominal in deterministic order.
name: getDeclaredMethods
order: 27
typeParameters: []
parameters: []
returns:
  type: vector<ReflectionMethod>
  description: Returns methods declared by this nominal in deterministic order.
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

Returns methods declared by this nominal in deterministic order.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
