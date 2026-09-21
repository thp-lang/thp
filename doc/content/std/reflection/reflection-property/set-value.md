---
kind: method
id: std.reflection.ReflectionProperty::setValue
title: ReflectionProperty::setValue
summary: Writes a checked value to the exact retained property slot.
name: setValue
order: 73
typeParameters: []
parameters:
  - name: receiver
    type: mixed
    description: Object receiver, or null for a static method.
  - name: value
    type: mixed
    description: Value checked against retained metadata.
returns:
  type: void
  description: Writes a checked value to the exact retained property slot.
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

Writes a checked value to the exact retained property slot.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
