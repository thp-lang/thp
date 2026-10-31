---
kind: method
id: std.reflection.ReflectionFunctionAbstract::getNumberOfRequiredParameters
title: ReflectionFunctionAbstract::getNumberOfRequiredParameters
summary: Returns the count before optional and variadic parameters.
name: getNumberOfRequiredParameters
order: 43
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the count before optional and variadic parameters.
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

Returns the count before optional and variadic parameters.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract)
