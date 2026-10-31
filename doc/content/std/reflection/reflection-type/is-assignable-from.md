---
kind: method
id: std.reflection.ReflectionType::isAssignableFrom
title: ReflectionType::isAssignableFrom
summary: Reports whether values of the other type are assignable to this type.
name: isAssignableFrom
order: 4
typeParameters: []
parameters:
  - name: other
    type: ReflectionType
    description: Value supplied to the operation.
returns:
  type: bool
  description: Reports whether values of the other type are assignable to this type.
errors: []
related:
  - std.reflection.ReflectionType
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionType
visibility: public
modifiers: []
---

Reports whether values of the other type are assignable to this type.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionType`](thp:std.reflection.ReflectionType)
