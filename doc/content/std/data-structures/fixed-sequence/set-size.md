---
kind: method
id: std.dataStructures.FixedSequence::setSize
title: FixedSequence::setSize
summary: "Changes the number of slots."
name: setSize
order: 3
typeParameters: []
parameters:
  - name: size
    type: "int"
    description: "Number of slots; must be non-negative."
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

Growing appends null slots. Shrinking discards values beyond the new size; retained positions keep their values. Zero empties the sequence. Existing iterator snapshots keep their original slots.

## See also

- [`FixedSequence`](thp:std.dataStructures.FixedSequence)
