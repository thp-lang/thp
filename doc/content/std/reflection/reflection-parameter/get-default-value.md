---
kind: method
id: std.reflection.ReflectionParameter::getDefaultValue
title: ReflectionParameter::getDefaultValue
summary: Returns a fresh materialization of the retained default.
name: getDefaultValue
order: 79
typeParameters: []
parameters: []
returns:
  type: mixed
  description: Returns a fresh materialization of the retained default.
errors:
  - type: ReflectionException
    description: The retained descriptor, receiver, arguments, or requested operation is invalid.
related:
  - std.reflection.ReflectionParameter
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionParameter
visibility: public
modifiers: []
---

Returns a fresh materialization of the retained default.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionParameter`](thp:std.reflection.ReflectionParameter)
