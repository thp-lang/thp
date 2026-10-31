---
kind: method
id: std.reflection.ReflectionMethod::isFinal
title: ReflectionMethod::isFinal
summary: Reports whether the declaration is final.
name: isFinal
order: 59
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the declaration is final.
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

Reports whether the declaration is final.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionMethod`](thp:std.reflection.ReflectionMethod)
