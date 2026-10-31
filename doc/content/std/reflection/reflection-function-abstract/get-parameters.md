---
kind: method
id: std.reflection.ReflectionFunctionAbstract::getParameters
title: ReflectionFunctionAbstract::getParameters
summary: Returns parameter descriptors in declaration order.
name: getParameters
order: 44
typeParameters: []
parameters: []
returns:
  type: vector<ReflectionParameter>
  description: Returns parameter descriptors in declaration order.
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

Returns parameter descriptors in declaration order.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionFunctionAbstract`](thp:std.reflection.ReflectionFunctionAbstract)
