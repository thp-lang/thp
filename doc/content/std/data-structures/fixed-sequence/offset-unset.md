---
kind: method
id: std.dataStructures.FixedSequence::offsetUnset
title: FixedSequence::offsetUnset
summary: "Clears a slot while preserving its position."
name: offsetUnset
order: 7
typeParameters: []
parameters:
  - name: offset
    type: "int"
    description: "Zero-based slot index."
returns:
  type: "void"
  description: "This method does not return a value."
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

Sets the slot to `null`. Size and all other positions remain unchanged, including when the slot was already null. Existing snapshots retain their previous values.

## See also

- [`FixedSequence`](thp:std.dataStructures.FixedSequence)
