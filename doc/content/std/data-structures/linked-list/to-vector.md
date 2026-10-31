---
kind: method
id: std.dataStructures.LinkedList::toVector
title: LinkedList::toVector
summary: "Returns a vector value snapshot."
name: toVector
order: 9
typeParameters: []
parameters: []
returns:
  type: "vector<T>"
  description: "Returns a vector value snapshot."
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

Preserves front-to-back storage order with dense zero-based offsets, including on `Stack<T>`. Later sequence mutations and writes to the returned copy-on-write vector do not alter each other. Object elements still share identity.

## See also

- [`LinkedList`](thp:std.dataStructures.LinkedList)
