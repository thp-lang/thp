---
kind: method
id: std.reflection.ReflectionClass::getNamespaceName
title: ReflectionClass::getNamespaceName
summary: Returns the nominal namespace without a leading separator.
name: getNamespaceName
order: 13
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the nominal namespace without a leading separator.
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

Returns the nominal namespace without a leading separator.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
