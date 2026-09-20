---
kind: method
id: std.reflection.ReflectionClass::getTraits
title: ReflectionClass::getTraits
summary: Returns directly composed trait descriptors in source order.
name: getTraits
order: 25
typeParameters: []
parameters: []
returns:
  type: vector<ReflectionClass>
  description: Returns directly composed trait descriptors in source order.
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

Returns directly composed trait descriptors in source order.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
