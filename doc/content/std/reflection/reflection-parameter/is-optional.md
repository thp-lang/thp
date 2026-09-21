---
kind: method
id: std.reflection.ReflectionParameter::isOptional
title: ReflectionParameter::isOptional
summary: Reports whether the parameter has a default or is variadic.
name: isOptional
order: 80
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the parameter has a default or is variadic.
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

Reports whether the parameter has a default or is variadic.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionParameter`](thp:std.reflection.ReflectionParameter)
