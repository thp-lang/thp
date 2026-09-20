---
kind: method
id: std.reflection.ReflectionClass::hasMethod
title: ReflectionClass::hasMethod
summary: Reports whether an effective method with the exact name exists.
name: hasMethod
order: 31
typeParameters: []
parameters:
  - name: name
    type: string
    description: Value supplied to the operation.
returns:
  type: bool
  description: Reports whether an effective method with the exact name exists.
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

Reports whether an effective method with the exact name exists.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
