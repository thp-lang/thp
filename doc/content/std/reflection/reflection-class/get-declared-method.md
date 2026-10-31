---
kind: method
id: std.reflection.ReflectionClass::getDeclaredMethod
title: ReflectionClass::getDeclaredMethod
summary: Returns an exact declared method or null.
name: getDeclaredMethod
order: 28
typeParameters: []
parameters:
  - name: name
    type: string
    description: Value supplied to the operation.
returns:
  type: ?ReflectionMethod
  description: Returns an exact declared method or null.
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

Returns an exact declared method or null.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
