---
kind: method
id: std.reflection.ReflectionFunction::__construct
title: ReflectionFunction::__construct
summary: Reflects a linked top-level function name.
name: __construct
order: 10
typeParameters: []
parameters:
  - name: function
    type: string
    description: Canonical top-level function name with an optional leading separator.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - type: TypeError
    description: A dynamic value supplied for the function name is not a string.
  - type: ReflectionException
    description: The name is invalid or the function is not in the linked program.
related:
  - std.reflection.ReflectionFunction
status: experimental
availability: implemented
notice: This descriptor constructor executes in the reference VM; closures remain unsupported.
version: "0.1"
owner: std.reflection.ReflectionFunction
visibility: public
modifiers: []
---

Lookup is case-sensitive and does not perform runtime autoloading.

## See also

- [`ReflectionFunction`](thp:std.reflection.ReflectionFunction)
