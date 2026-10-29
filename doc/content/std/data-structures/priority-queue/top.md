---
kind: method
id: std.dataStructures.PriorityQueue::top
title: PriorityQueue::top
summary: "Reads the highest-priority value."
name: top
order: 2
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Reads the highest-priority value."
errors:
  - type: UnderflowException
    description: "The collection is empty."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.PriorityQueue
visibility: public
modifiers: []
---

Equal priorities retain insertion order. The result is the value alone, without its priority. Does not consume or change the collection.

## See also

- [`PriorityQueue`](thp:std.dataStructures.PriorityQueue)
