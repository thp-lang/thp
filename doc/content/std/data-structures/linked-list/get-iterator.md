---
kind: method
id: std.dataStructures.LinkedList::getIterator
title: LinkedList::getIterator
summary: "Captures a sequence traversal snapshot."
name: getIterator
order: 10
typeParameters: []
parameters: []
returns:
  type: "Traversable<int, T>"
  description: "Captures a sequence traversal snapshot."
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

Returns a fresh [`VectorIterator`](thp:std.spl.VectorIterator), normally front to back: `LinkedList<T>` and `Queue<T>` traverse in FIFO order. On `Stack<T>`, this inherited method reverses the captured values for top-to-bottom LIFO traversal. Keys are dense zero-based offsets in traversal order. The cursor does not consume the sequence; later mutations do not change it. Object elements retain identity. Each `foreach` obtains a fresh cursor.

## See also

- [`LinkedList`](thp:std.dataStructures.LinkedList)
