---
kind: method
id: std.dataStructures.LinkedList::bottom
title: LinkedList::bottom
summary: "Reads the first value."
name: bottom
order: 6
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Reads the first value."
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

Returns the value without changing the sequence.

## See also

- [`LinkedList`](thp:std.dataStructures.LinkedList)
