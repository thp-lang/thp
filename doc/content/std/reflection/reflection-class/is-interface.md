---
kind: method
id: std.reflection.ReflectionClass::isInterface
title: ReflectionClass::isInterface
summary: Reports whether the nominal is an interface.
name: isInterface
order: 18
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the nominal is an interface.
errors: []
related:
  - std.reflection.ReflectionClass
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionClass
visibility: public
modifiers: []
---

Reports whether the nominal is an interface.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
