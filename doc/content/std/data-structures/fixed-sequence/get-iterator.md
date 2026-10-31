---
kind: method
id: std.dataStructures.FixedSequence::getIterator
title: FixedSequence::getIterator
summary: "Captures a snapshot of all slots."
name: getIterator
order: 8
typeParameters: []
parameters: []
returns:
  type: "Traversable<int, ?T>"
  description: "Captures a snapshot of all slots."
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

Returns a fresh [`VectorIterator`](thp:std.spl.VectorIterator) with zero-based keys, including null slots. Later writes, clears, and resizing do not change the captured collection. Each `foreach` obtains a fresh cursor. Object values retain their identity; this is not a deep copy.

## See also

- [`FixedSequence`](thp:std.dataStructures.FixedSequence)
