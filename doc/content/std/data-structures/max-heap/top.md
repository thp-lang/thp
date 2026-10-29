---
kind: method
id: std.dataStructures.MaxHeap::top
title: MaxHeap::top
summary: "Reads the greatest value."
name: top
order: 2
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Reads the greatest value."
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

Does not consume or change the collection.

## See also

- [`MaxHeap`](thp:std.dataStructures.MaxHeap)
