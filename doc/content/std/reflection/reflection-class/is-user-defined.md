---
kind: method
id: std.reflection.ReflectionClass::isUserDefined
title: ReflectionClass::isUserDefined
summary: Reports whether the nominal came from linked user source.
name: isUserDefined
order: 21
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the nominal came from linked user source.
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

Reports whether the nominal came from linked user source.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
