---
kind: method
id: std.reflection.ReflectionProperty::getDefaultValue
title: ReflectionProperty::getDefaultValue
summary: Returns the retained default or null when none exists.
name: getDefaultValue
order: 67
typeParameters: []
parameters: []
returns:
  type: mixed
  description: Returns a fresh materialization of the retained default, or null.
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

Returns a fresh materialization of the retained default, or `null` when the property has no default.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
