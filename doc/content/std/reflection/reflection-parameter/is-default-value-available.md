---
kind: method
id: std.reflection.ReflectionParameter::isDefaultValueAvailable
title: ReflectionParameter::isDefaultValueAvailable
summary: Reports whether a constant default is retained.
name: isDefaultValueAvailable
order: 78
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether a constant default is retained.
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

Reports whether a constant default is retained.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionParameter`](thp:std.reflection.ReflectionParameter)
