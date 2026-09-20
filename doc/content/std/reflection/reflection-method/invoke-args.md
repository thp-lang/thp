---
kind: method
id: std.reflection.ReflectionMethod::invokeArgs
title: ReflectionMethod::invokeArgs
summary: Invokes the exact retained method with a checked receiver and arguments.
name: invokeArgs
order: 61
typeParameters: []
parameters:
  - name: receiver
    type: mixed
    description: Compatible object receiver; ignored for a static method.
  - name: arguments
    type: vector<mixed>|map<string, mixed>
    description: Positional vector or case-sensitive named map.
    default: "[]"
returns:
  type: mixed
  description: Invokes the exact retained method with a checked receiver and arguments.
errors:
  - type: ReflectionException
    description: The descriptor or instance receiver is invalid, or the method is abstract.
  - type: TypeError
    description: A dynamic argument has the wrong type.
  - type: ArgumentCountError
    description: A required argument is missing or too many positional arguments were supplied.
  - type: Error
    description: A named argument is unknown, duplicated, or targets a variadic parameter.
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

Invokes the exact retained method with a checked receiver and arguments. Static methods ignore the receiver, and constructor descriptors may initialize compatible existing objects.

Reflection lookups are case-sensitive and operate only on metadata retained in the verified linked program. Descriptor state cannot be mutated.

## See also

- [`ReflectionMethod`](thp:std.reflection.ReflectionMethod)
