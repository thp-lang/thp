---
kind: method
id: std.dataStructures.MaxHeap::getIterator
title: MaxHeap::getIterator
summary: "Captures an ordered traversal snapshot."
name: getIterator
order: 6
typeParameters: []
parameters: []
returns:
  type: "Traversable<int, T>"
  description: "Captures an ordered traversal snapshot."
errors: []
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.MaxHeap
visibility: public
modifiers: []
---

Returns a fresh [`VectorIterator`](thp:std.spl.VectorIterator) in greatest-first order with dense zero-based keys. An empty collection produces an empty cursor. Iteration does not consume the collection and later insertions or extractions do not change the snapshot. Object values retain identity. Each `foreach` obtains a fresh cursor.

Current implementation detail: ordering repeatedly scans the remaining values, taking quadratic time and allocating a snapshot; this cost is not an API guarantee.

## See also

- [`MaxHeap`](thp:std.dataStructures.MaxHeap)
