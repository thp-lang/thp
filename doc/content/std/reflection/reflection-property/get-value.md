---
kind: method
id: std.reflection.ReflectionProperty::getValue
title: ReflectionProperty::getValue
summary: Reads the exact retained property slot from a compatible object.
name: getValue
order: 72
typeParameters: []
parameters:
  - name: receiver
    type: mixed
    description: Object receiver, or null for a static method.
returns:
  type: mixed
  description: Reads the exact retained property slot from a compatible object.
errors:
  - type: ReflectionException
    description: The retained descriptor, receiver, arguments, or requested operation is invalid.
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

Reads the exact retained property slot from a compatible object.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
