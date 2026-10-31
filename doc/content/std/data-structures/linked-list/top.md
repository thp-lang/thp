---
kind: method
id: std.dataStructures.LinkedList::top
title: LinkedList::top
summary: "Reads the last value."
name: top
order: 5
typeParameters: []
parameters: []
returns:
  type: "T"
  description: "Reads the last value."
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
