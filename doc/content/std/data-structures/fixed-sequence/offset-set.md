---
kind: method
id: std.dataStructures.FixedSequence::offsetSet
title: FixedSequence::offsetSet
summary: "Writes an existing slot."
name: offsetSet
order: 6
typeParameters: []
parameters:
  - name: offset
    type: "?int"
    description: "Existing slot index; null append is unsupported."
  - name: value
    type: "?T"
    description: "Value to store, or null to clear the slot."
returns:
  type: "void"
  description: "This method does not return a value."
errors:
  - type: ValueError
    description: "The offset is null."
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

Replaces the slot without changing size or positions. A null value clears it; a null offset does not append. Existing snapshots retain their previous slot values.

## See also

- [`FixedSequence`](thp:std.dataStructures.FixedSequence)
