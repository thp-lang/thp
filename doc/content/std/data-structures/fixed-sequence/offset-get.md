---
kind: method
id: std.dataStructures.FixedSequence::offsetGet
title: FixedSequence::offsetGet
summary: "Reads a nullable slot."
name: offsetGet
order: 5
typeParameters: []
parameters:
  - name: offset
    type: "int"
    description: "Zero-based slot index."
returns:
  type: "?T"
  description: "Reads a nullable slot."
errors:
  - type: OutOfBoundsException
    description: "The offset is negative or at least the current size."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.FixedSequence
visibility: public
modifiers: []
---

Returns the stored value or `null` for a cleared or never initialized slot. Reading does not move or remove any slot.

## See also

- [`FixedSequence`](thp:std.dataStructures.FixedSequence)
