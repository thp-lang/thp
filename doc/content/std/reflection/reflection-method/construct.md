---
kind: method
id: std.reflection.ReflectionMethod::__construct
title: ReflectionMethod::__construct
summary: Reflects a method on an object or linked nominal.
name: __construct
order: 10
typeParameters: []
parameters:
  - name: objectOrClass
    type: object|string
    description: Object instance or canonical nominal name.
  - name: method
    type: string
    description: Exact case-sensitive method name.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - type: TypeError
    description: The target is neither an object nor a string, or the method name is not a string.
  - type: ReflectionException
    description: The target or method does not exist in the linked program.
related:
  - std.reflection.ReflectionMethod
status: experimental
availability: implemented
notice: This two-argument descriptor constructor executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionMethod
visibility: public
modifiers: []
---

Object inputs retain concrete generic arguments. The deprecated one-argument PHP form is not supported.

## See also

- [`ReflectionMethod`](thp:std.reflection.ReflectionMethod)
