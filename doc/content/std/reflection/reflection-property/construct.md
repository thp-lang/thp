---
kind: method
id: std.reflection.ReflectionProperty::__construct
title: ReflectionProperty::__construct
summary: Reflects a property on an object or linked nominal.
name: __construct
order: 10
typeParameters: []
parameters:
  - name: objectOrClass
    type: object|string
    description: Object instance or canonical nominal name.
  - name: property
    type: string
    description: Exact case-sensitive property name.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - type: TypeError
    description: The target is neither an object nor a string, or the property name is not a string.
  - type: ReflectionException
    description: The target or property does not exist in the linked program.
related:
  - std.reflection.ReflectionProperty
status: experimental
availability: implemented
notice: This descriptor constructor executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionProperty
visibility: public
modifiers: []
---

Object inputs retain concrete generic arguments. Lookup is case-sensitive and does not perform runtime autoloading.

## See also

- [`ReflectionProperty`](thp:std.reflection.ReflectionProperty)
