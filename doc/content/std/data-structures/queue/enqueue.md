---
kind: method
id: std.dataStructures.Queue::enqueue
title: Queue::enqueue
summary: "Appends a value to the queue."
name: enqueue
order: 0
typeParameters: []
parameters:
  - name: value
    type: "T"
    description: "Value to append."
returns:
  type: "void"
  description: "This method does not return a value."
errors: []
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.Queue
visibility: public
modifiers: []
---

Equivalent to inherited [`push()`](thp:std.dataStructures.LinkedList::push). Later dequeues remove older values first; existing snapshots remain unchanged.

## See also

- [`Queue`](thp:std.dataStructures.Queue)
