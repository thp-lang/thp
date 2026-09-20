---
kind: method
id: std.reflection.ReflectionParameter::__construct
title: ReflectionParameter::__construct
summary: Reflects a parameter by position or name.
name: __construct
order: 10
typeParameters: []
parameters:
  - name: function
    type: string|vector<mixed>
    description: Function name or exact two-value method callable.
  - name: parameter
    type: int|string
    description: Zero-based position or exact parameter name.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - type: TypeError
    description: The callable shape or selector type is invalid.
  - type: ReflectionException
    description: The callable or parameter does not exist in the linked program.
related:
  - std.reflection.ReflectionParameter
status: experimental
availability: implemented
notice: This descriptor constructor executes in the reference VM; closures and invokable objects remain unsupported.
version: "0.1"
owner: std.reflection.ReflectionParameter
visibility: public
modifiers: []
---

A method callable must be exactly `[object|string, string]`. Object targets retain concrete generic arguments.

## See also

- [`ReflectionParameter`](thp:std.reflection.ReflectionParameter)
