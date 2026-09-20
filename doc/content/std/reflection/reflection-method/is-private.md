---
kind: method
id: std.reflection.ReflectionMethod::isPrivate
title: ReflectionMethod::isPrivate
summary: Reports whether the declaration is private.
name: isPrivate
order: 56
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the declaration is private.
errors: []
related:
  - std.reflection.ReflectionMethod
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionMethod
visibility: public
modifiers: []
---

Reports whether the declaration is private.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionMethod`](thp:std.reflection.ReflectionMethod)
