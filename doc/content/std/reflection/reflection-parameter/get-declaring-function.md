---
kind: method
id: std.reflection.ReflectionParameter::getDeclaringFunction
title: ReflectionParameter::getDeclaringFunction
summary: Returns the function or method descriptor that declared the parameter.
name: getDeclaringFunction
order: 77
typeParameters: []
parameters: []
returns:
  type: ReflectionFunctionAbstract
  description: Returns the function or method descriptor that declared the parameter.
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

Returns the function or method descriptor that declared the parameter.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionParameter`](thp:std.reflection.ReflectionParameter)
