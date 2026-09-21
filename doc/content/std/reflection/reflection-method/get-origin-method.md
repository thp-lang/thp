---
kind: method
id: std.reflection.ReflectionMethod::getOriginMethod
title: ReflectionMethod::getOriginMethod
summary: Returns the ultimate trait method descriptor, if present.
name: getOriginMethod
order: 53
typeParameters: []
parameters: []
returns:
  type: ?ReflectionMethod
  description: Returns the ultimate trait method descriptor, if present.
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

Returns the ultimate trait method descriptor, if present.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionMethod`](thp:std.reflection.ReflectionMethod)
