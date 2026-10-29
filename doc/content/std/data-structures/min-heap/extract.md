---
kind: method
id: std.dataStructures.MinHeap::extract
title: MinHeap::extract
summary: "Removes and returns the least value."
name: extract
order: 3
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Removes and returns the least value."
errors:
  - type: UnderflowException
    description: "The collection is empty."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.MinHeap
visibility: public
modifiers: []
---

Decreases the count by one; existing snapshots remain unchanged.

## See also

- [`MinHeap`](thp:std.dataStructures.MinHeap)
