---
kind: method
id: std.reflection.ReflectionProperty::isStatic
title: ReflectionProperty::isStatic
summary: Reports whether the declaration is static.
name: isStatic
order: 71
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the declaration is static.
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

Reports whether the declaration is static.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
