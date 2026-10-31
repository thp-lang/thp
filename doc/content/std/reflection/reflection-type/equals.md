---
kind: method
id: std.reflection.ReflectionType::equals
title: ReflectionType::equals
summary: Compares two reflected types semantically.
name: equals
order: 3
typeParameters: []
parameters:
  - name: other
    type: ReflectionType
    description: Value supplied to the operation.
returns:
  type: bool
  description: Compares two reflected types semantically.
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

Compares two reflected types semantically.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionType`](thp:std.reflection.ReflectionType)
