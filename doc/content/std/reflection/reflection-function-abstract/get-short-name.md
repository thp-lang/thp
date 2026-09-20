---
kind: method
id: std.reflection.ReflectionFunctionAbstract::getShortName
title: ReflectionFunctionAbstract::getShortName
summary: Returns the unqualified callable name.
name: getShortName
order: 39
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the unqualified callable name.
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

Returns the unqualified callable name.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract)
