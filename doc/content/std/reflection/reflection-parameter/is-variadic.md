---
kind: method
id: std.reflection.ReflectionParameter::isVariadic
title: ReflectionParameter::isVariadic
summary: Reports whether the parameter captures remaining positional arguments.
name: isVariadic
order: 81
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the parameter captures remaining positional arguments.
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

Reports whether the parameter captures remaining positional arguments.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionParameter`](thp:std.reflection.ReflectionParameter)
