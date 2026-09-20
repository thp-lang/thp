---
kind: method
id: std.reflection.ReflectionFunction::invokeArgs
title: ReflectionFunction::invokeArgs
summary: Invokes the exact function with checked positional or named arguments.
name: invokeArgs
order: 50
typeParameters: []
parameters:
  - name: arguments
    type: vector<mixed>|map<string, mixed>
    description: Positional vector or case-sensitive named map.
    default: "[]"
returns:
  type: mixed
  description: Invokes the exact function with checked positional or named arguments.
errors:
  - type: ReflectionException
    description: The retained callable descriptor is invalid.
  - type: TypeError
    description: A dynamic argument has the wrong type.
  - type: ArgumentCountError
    description: A required argument is missing or too many positional arguments were supplied.
  - type: Error
    description: A named argument is unknown, duplicated, or targets a variadic parameter.
related:
  - std.reflection.ReflectionFunction
status: experimental
availability: implemented
notice: This descriptor operation executes in the reference VM.
version: "0.1"
owner: std.reflection.ReflectionFunction
visibility: public
modifiers: []
---

Invokes the exact function with checked positional or named arguments.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionFunction`](thp:std.reflection.ReflectionFunction)
