---
kind: method
id: std.reflection.ReflectionMethod::isConstructor
title: ReflectionMethod::isConstructor
summary: Reports whether this is the __construct declaration.
name: isConstructor
order: 60
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether this is the __construct declaration.
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

Reports whether this is the __construct declaration.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionMethod`](thp:std.reflection.ReflectionMethod)
