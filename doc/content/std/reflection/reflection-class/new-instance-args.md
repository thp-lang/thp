---
kind: method
id: std.reflection.ReflectionClass::newInstanceArgs
title: ReflectionClass::newInstanceArgs
summary: Allocates and constructs an instance with positional or named arguments.
name: newInstanceArgs
order: 37
typeParameters: []
parameters:
  - name: arguments
    type: vector<mixed>|map<string, mixed>
    description: Positional vector or case-sensitive named map.
    default: "[]"
returns:
  type: mixed
  description: Allocates and constructs an instance with positional or named arguments.
errors:
  - type: ReflectionException
    description: The reflected constructor is missing, non-public, or invalid.
  - type: TypeError
    description: A dynamic argument has the wrong type.
  - type: ArgumentCountError
    description: A required argument is missing or too many positional arguments were supplied.
  - type: Error
    description: The class is not constructible or a named argument is invalid.
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

Allocates and constructs an instance with positional or named arguments. Only public constructors are accepted.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionClass`](thp:std.reflection.ReflectionClass)
