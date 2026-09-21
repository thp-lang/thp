---
kind: method
id: std.reflection.ReflectionProperty::getOriginTrait
title: ReflectionProperty::getOriginTrait
summary: Returns the ultimate trait descriptor, if composed from a trait.
name: getOriginTrait
order: 64
typeParameters: []
parameters: []
returns:
  type: ?ReflectionClass
  description: Returns the ultimate trait descriptor, if composed from a trait.
errors: []
related:
  - std.reflection.ReflectionProperty
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionProperty
visibility: public
modifiers: []
---

Returns the ultimate trait descriptor, if composed from a trait.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
