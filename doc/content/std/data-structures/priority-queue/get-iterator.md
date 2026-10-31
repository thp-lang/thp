---
kind: method
id: std.dataStructures.PriorityQueue::getIterator
title: PriorityQueue::getIterator
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
owner: std.dataStructures.PriorityQueue
visibility: public
modifiers: []
---

Returns a fresh [`VectorIterator`](thp:std.spl.VectorIterator) in descending integer priority order with dense zero-based keys. Equal priorities retain insertion order; only values are yielded. An empty collection produces an empty cursor. Iteration does not consume the collection and later insertions or extractions do not change the snapshot. Object values retain identity. Each `foreach` obtains a fresh cursor.

Current implementation detail: ordering repeatedly scans the remaining values, taking quadratic time and allocating a snapshot; this cost is not an API guarantee.

## See also

- [`PriorityQueue`](thp:std.dataStructures.PriorityQueue)
