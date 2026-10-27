---
kind: method
id: std.dataStructures.PriorityQueue::insert
title: PriorityQueue::insert
summary: "Inserts a value with an integer priority."
name: insert
order: 1
typeParameters: []
parameters:
  - name: value
    type: "T"
    description: "Value to store."
  - name: priority
    type: "int"
    description: "Integer priority; larger values are extracted first."
returns:
  type: "void"
  description: "This method does not return a value."
errors:
  - type: OverflowException
    description: "The insertion sequence counter cannot be incremented."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.PriorityQueue
visibility: public
modifiers: []
---

Negative priorities are accepted. Equal priorities retain insertion order. The stored values are not compared, so `T` need not be comparable. Existing snapshots remain unchanged.

## See also

- [`PriorityQueue`](thp:std.dataStructures.PriorityQueue)
