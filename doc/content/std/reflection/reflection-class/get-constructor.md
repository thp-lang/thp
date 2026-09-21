---
kind: method
id: std.reflection.ReflectionClass::getConstructor
title: ReflectionClass::getConstructor
summary: Returns the effective constructor descriptor, if present.
name: getConstructor
order: 26
typeParameters: []
parameters: []
returns:
  type: ?ReflectionMethod
  description: Returns the effective constructor descriptor, if present.
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

Returns the effective constructor descriptor, if present.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
