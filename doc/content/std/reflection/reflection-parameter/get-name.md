---
kind: method
id: std.reflection.ReflectionParameter::getName
title: ReflectionParameter::getName
summary: Returns the case-sensitive parameter name.
name: getName
order: 74
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the case-sensitive parameter name.
errors: []
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

Returns the case-sensitive parameter name.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionParameter`](thp:std.reflection.ReflectionParameter)
