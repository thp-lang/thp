---
kind: method
id: std.reflection.ReflectionClass::getProperty
title: ReflectionClass::getProperty
summary: Returns an exact effective property.
name: getProperty
order: 35
typeParameters: []
parameters:
  - name: name
    type: string
    description: Value supplied to the operation.
returns:
  type: ReflectionProperty
  description: Returns the exact effective property.
errors:
  - type: ReflectionException
    description: No effective property has the exact supplied name.
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

Returns an exact effective property or throws `ReflectionException` when absent.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
