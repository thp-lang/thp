---
kind: method
id: std.reflection.ReflectionFunctionAbstract::getNamespaceName
title: ReflectionFunctionAbstract::getNamespaceName
summary: Returns the callable namespace without a leading separator.
name: getNamespaceName
order: 40
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the callable namespace without a leading separator.
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

Returns the callable namespace without a leading separator.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract)
