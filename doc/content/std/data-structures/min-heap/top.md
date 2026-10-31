---
kind: method
id: std.dataStructures.MinHeap::top
title: MinHeap::top
summary: "Reads the least value."
name: top
order: 2
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Reads the least value."
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

Does not consume or change the collection.

## See also

- [`MinHeap`](thp:std.dataStructures.MinHeap)
