---
kind: method
id: std.reflection.ReflectionClass::getModuleName
title: ReflectionClass::getModuleName
summary: Returns the canonical owning module name.
name: getModuleName
order: 14
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the canonical owning module name.
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

Returns the canonical owning module name.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
