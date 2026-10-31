---
kind: method
id: std.reflection.ReflectionUnionType::getTypes
title: ReflectionUnionType::getTypes
summary: Returns normalized union members in deterministic order.
name: getTypes
order: 9
typeParameters: []
parameters: []
returns:
  type: vector<ReflectionType>
  description: Returns normalized union members in deterministic order.
errors: []
related:
  - std.reflection.ReflectionUnionType
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionUnionType
visibility: public
modifiers: []
---

Returns normalized union members in deterministic order.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionUnionType`](thp:std.reflection.ReflectionUnionType)
