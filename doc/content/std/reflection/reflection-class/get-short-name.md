---
kind: method
id: std.reflection.ReflectionClass::getShortName
title: ReflectionClass::getShortName
summary: Returns the unqualified nominal name.
name: getShortName
order: 12
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the unqualified nominal name.
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

Returns the unqualified nominal name.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
