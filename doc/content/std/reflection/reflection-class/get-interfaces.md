---
kind: method
id: std.reflection.ReflectionClass::getInterfaces
title: ReflectionClass::getInterfaces
summary: Returns direct and retained interface descriptors in deterministic order.
name: getInterfaces
order: 24
typeParameters: []
parameters: []
returns:
  type: vector<ReflectionClass>
  description: Returns direct and retained interface descriptors in deterministic order.
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

Returns direct and retained interface descriptors in deterministic order.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
