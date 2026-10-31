---
kind: method
id: std.reflection.ReflectionMethod::getOriginTrait
title: ReflectionMethod::getOriginTrait
summary: Returns the ultimate trait descriptor, if composed from a trait.
name: getOriginTrait
order: 52
typeParameters: []
parameters: []
returns:
  type: ?ReflectionClass
  description: Returns the ultimate trait descriptor, if composed from a trait.
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

Returns the ultimate trait descriptor, if composed from a trait.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionMethod`](thp:std.reflection.ReflectionMethod)
