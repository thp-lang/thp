---
kind: method
id: std.reflection.ReflectionProperty::isProtected
title: ReflectionProperty::isProtected
summary: Reports whether the declaration is protected.
name: isProtected
order: 69
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the declaration is protected.
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

Reports whether the declaration is protected.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
