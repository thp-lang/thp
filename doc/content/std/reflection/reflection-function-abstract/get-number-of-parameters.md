---
kind: method
id: std.reflection.ReflectionFunctionAbstract::getNumberOfParameters
title: ReflectionFunctionAbstract::getNumberOfParameters
summary: Returns the total parameter count.
name: getNumberOfParameters
order: 42
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the total parameter count.
errors: []
related:
  - std.reflection.ReflectionFunctionAbstract
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionFunctionAbstract
visibility: public
modifiers: []
---

Returns the total parameter count.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract)
