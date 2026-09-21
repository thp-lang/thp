---
kind: method
id: std.reflection.ReflectionClass::getMethods
title: ReflectionClass::getMethods
summary: Returns effective methods with declared methods first.
name: getMethods
order: 29
typeParameters: []
parameters: []
returns:
  type: vector<ReflectionMethod>
  description: Returns effective methods with declared methods first.
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

Returns effective methods with declared methods first.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
