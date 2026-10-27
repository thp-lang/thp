---
kind: method
id: std.dataStructures.FixedSequence::__construct
title: FixedSequence::__construct
summary: "Creates nullable slots with an explicit size."
name: __construct
order: 0
typeParameters: []
parameters:
  - name: size
    type: "int"
    description: "Number of slots; must be non-negative."
    default: "0"
returns:
  type: "void"
  description: "This method does not return a value."
errors:
  - type: ValueError
    description: "The size is negative."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.FixedSequence
visibility: public
modifiers: []
---

All slots initially contain `null`. The default size is zero. Specify `T` explicitly when it cannot be inferred from context.

## See also

- [`FixedSequence`](thp:std.dataStructures.FixedSequence)
