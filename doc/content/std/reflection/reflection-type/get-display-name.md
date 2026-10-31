---
kind: method
id: std.reflection.ReflectionType::getDisplayName
title: ReflectionType::getDisplayName
summary: Returns the normalized semantic type spelling.
name: getDisplayName
order: 2
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the normalized semantic type spelling.
errors: []
related:
  - std.reflection.ReflectionType
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionType
visibility: public
modifiers: []
---

Returns the normalized semantic type spelling.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionType`](thp:std.reflection.ReflectionType)
