---
kind: method
id: std.baseTypes.TraceLine::__construct
title: TraceLine::__construct
summary: Initializes an immutable trace frame from its seven fields.
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
    type: mixed
    description: An object or null receiver; other values throw InvalidArgumentException.
  - name: type
    type: string
    description: 'Call operator: "->", "::", or "".'
  - name: args
    type: mixed
    description: A vector or null; other values throw InvalidArgumentException.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description: InvalidArgumentException when line, receiver, call operator, or arguments are invalid.
related: []
status: experimental
availability: implemented
notice: This method executes in the reference VM.
version: "0.1"
owner: std.baseTypes.TraceLine
visibility: public
modifiers: []
---

[`TraceLine`](thp:std.baseTypes.TraceLine)`::__construct()` initializes a trace frame.

## Behavior

The line must be non-negative. The receiver must be an object or `null`; the
arguments must be a vector or `null`. The call operator is `"->"`, `"::"`, or
`""`. The seven fields are read-only after construction.

## Example

```thp
$instance = new TraceLine($function, $line, $file, $class, $object, $type, $args);
```

The call uses the signature and defaults documented above.

## See also

- [`TraceLine`](thp:std.baseTypes.TraceLine)
