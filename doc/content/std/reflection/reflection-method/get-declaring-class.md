---
kind: method
id: std.reflection.ReflectionMethod::getDeclaringClass
title: ReflectionMethod::getDeclaringClass
summary: Returns the consuming or declaring class.
name: getDeclaringClass
order: 51
typeParameters: []
parameters: []
returns:
  type: ReflectionClass
  description: Returns the consuming or declaring class.
errors: []
related:
  - std.reflection.ReflectionMethod
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionMethod
visibility: public
modifiers: []
---

Returns the consuming or declaring class.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionMethod`](thp:std.reflection.ReflectionMethod)
