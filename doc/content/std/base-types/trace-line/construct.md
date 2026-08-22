---
kind: method
id: std.baseTypes.TraceLine::__construct
title: TraceLine::__construct
summary:
  vector preserves argument order and permits values of any type. A map would
  incorrectly imply that trace arguments are primarily accessed by key.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: function
    type: string
    description: Callable name recorded for this frame.
  - name: line
    type: int
    description: Source or call-site line recorded for this frame.
  - name: file
    type: string
    description: Source file recorded for this frame.
  - name: class
    type: string
    description: Declaring class for a method frame.
  - name: object
    type: ?object
    description: Receiver recorded for an instance-method frame.
  - name: type
    type: string
    description: 'Call operator: "->", "::", or "".'
  - name: args
    type: ?vector<mixed>
    description: Ordered arguments when argument capture is enabled.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Construction fails when an argument violates the documented contract or an
      underlying resource cannot be created. Concrete THP error classes remain
      experimental unless named above.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.baseTypes.TraceLine
visibility: public
modifiers: []
---

[`TraceLine`](thp:std.baseTypes.TraceLine)`::__construct()` vector preserves argument order and permits values of any type. A map would incorrectly imply that trace arguments are primarily accessed by key.

## Behavior

vector preserves argument order and permits values of any type. A map would incorrectly imply that trace arguments are primarily accessed by key.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new TraceLine($function, $line, $file, $class, $object, $type, $args);
```

The call uses the signature and defaults documented above.

## See also

- [`TraceLine`](thp:std.baseTypes.TraceLine)
