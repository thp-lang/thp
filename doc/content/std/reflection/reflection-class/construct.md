---
kind: method
id: std.reflection.ReflectionClass::__construct
title: ReflectionClass::__construct
summary: Reflects an object or a linked nominal name.
name: __construct
order: 10
typeParameters: []
parameters:
  - name: objectOrClass
    type: object|string
    description: Object instance or canonical class, interface, or trait name.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - type: TypeError
    description: The value is neither an object nor a string.
  - type: ReflectionException
    description: The name is invalid or the nominal is not in the linked program.
related:
  - std.reflection.ReflectionClass
status: experimental
availability: implemented
notice: This descriptor constructor executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionClass
visibility: public
modifiers: []
---

Object inputs retain concrete generic arguments. String inputs accept one optional leading `\\`; generic class descriptors created by name are erased.

Lookup is case-sensitive and does not perform runtime autoloading.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
