---
kind: method
id: std.dataStructures.FixedSequence::offsetExists
title: FixedSequence::offsetExists
summary: "Tests whether a slot is in range and non-null."
name: offsetExists
order: 4
typeParameters: []
parameters:
  - name: offset
    type: "int"
    description: "Zero-based slot index."
returns:
  type: "bool"
  description: "Tests whether a slot is in range and non-null."
errors: []
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.FixedSequence
visibility: public
modifiers: []
---

Returns `false` for a negative or out-of-range offset and for a null slot. This does not distinguish a cleared slot from an invalid position; compare the offset with [`getSize()`](thp:std.dataStructures.FixedSequence::getSize) when that distinction matters.

## See also

- [`FixedSequence`](thp:std.dataStructures.FixedSequence)
