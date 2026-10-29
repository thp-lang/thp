---
kind: method
id: std.dataStructures.LinkedList::pop
title: LinkedList::pop
summary: "Removes and returns the last value."
name: pop
order: 3
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Removes and returns the last value."
errors:
  - type: UnderflowException
    description: "The sequence is empty."
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.LinkedList
visibility: public
modifiers: []
---

Removes one value and decreases the count. Existing snapshots remain unchanged.

## See also

- [`LinkedList`](thp:std.dataStructures.LinkedList)
