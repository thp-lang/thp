---
kind: method
id: std.reflection.ReflectionFunctionAbstract::isUserDefined
title: ReflectionFunctionAbstract::isUserDefined
summary: Reports whether the callable came from linked user source.
name: isUserDefined
order: 48
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the callable came from linked user source.
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

Reports whether the callable came from linked user source.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract)
