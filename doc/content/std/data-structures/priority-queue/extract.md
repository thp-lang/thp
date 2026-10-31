---
kind: method
id: std.dataStructures.PriorityQueue::extract
title: PriorityQueue::extract
summary: "Removes and returns the highest-priority value."
name: extract
order: 3
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Removes and returns the highest-priority value."
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

Equal priorities retain insertion order. The result is the value alone, without its priority. Decreases the count by one; existing snapshots remain unchanged.

## See also

- [`PriorityQueue`](thp:std.dataStructures.PriorityQueue)
