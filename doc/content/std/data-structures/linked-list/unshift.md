---
kind: method
id: std.dataStructures.LinkedList::unshift
title: LinkedList::unshift
summary: "Prepends a value."
name: unshift
order: 2
typeParameters: []
parameters:
  - name: value
    type: "T"
    description: "Value to store."
returns:
  type: "void"
  description: "This method does not return a value."
errors: []
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.LinkedList
visibility: public
modifiers: []
---

Adds the value at the front. The sequence grows by one; iterator snapshots already obtained remain unchanged.

## See also

- [`LinkedList`](thp:std.dataStructures.LinkedList)
