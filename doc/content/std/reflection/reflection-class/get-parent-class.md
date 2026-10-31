---
kind: method
id: std.reflection.ReflectionClass::getParentClass
title: ReflectionClass::getParentClass
summary: Returns the instantiated direct parent descriptor, if any.
name: getParentClass
order: 23
typeParameters: []
parameters: []
returns:
  type: ?ReflectionClass
  description: Returns the instantiated direct parent descriptor, if any.
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

Returns the instantiated direct parent descriptor, if any.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
