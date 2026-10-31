---
kind: method
id: std.reflection.ReflectionClass::isInstantiable
title: ReflectionClass::isInstantiable
summary: Reports whether reflective construction is available.
name: isInstantiable
order: 22
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether reflective construction is available.
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

Reports whether reflective construction is available.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
