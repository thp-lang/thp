---
kind: method
id: std.reflection.ReflectionProperty::hasDefaultValue
title: ReflectionProperty::hasDefaultValue
summary: Reports whether a constant default is retained.
name: hasDefaultValue
order: 66
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether a constant default is retained.
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

Reports whether a constant default is retained.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
