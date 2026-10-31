---
kind: method
id: std.reflection.ReflectionProperty::getName
title: ReflectionProperty::getName
summary: Returns the declared property name.
name: getName
order: 62
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the declared property name.
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

Returns the declared property name.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
