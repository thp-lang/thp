---
kind: method
id: std.dataStructures.MaxHeap::extract
title: MaxHeap::extract
summary: "Removes and returns the greatest value."
name: extract
order: 3
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Removes and returns the greatest value."
errors:
  - type: UnderflowException
    description: "The collection is empty."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.MaxHeap
visibility: public
modifiers: []
---

Decreases the count by one; existing snapshots remain unchanged.

## See also

- [`MaxHeap`](thp:std.dataStructures.MaxHeap)
