---
kind: method
id: std.dataStructures.Queue::dequeue
title: Queue::dequeue
summary: "Removes and returns the oldest value."
name: dequeue
order: 1
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Removes and returns the oldest value."
errors:
  - type: UnderflowException
    description: "The queue is empty."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.Queue
visibility: public
modifiers: []
---

Equivalent to inherited [`shift()`](thp:std.dataStructures.LinkedList::shift). Decreases the count by one without changing existing snapshots.

## See also

- [`Queue`](thp:std.dataStructures.Queue)
