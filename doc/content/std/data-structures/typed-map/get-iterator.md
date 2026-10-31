---
kind: method
id: std.dataStructures.TypedMap::getIterator
title: TypedMap::getIterator
summary: "Captures an insertion-ordered map snapshot."
name: getIterator
order: 7
typeParameters: []
parameters: []
returns:
  type: "Traversable<K, V>"
  description: "Captures an insertion-ordered map snapshot."
errors: []
related: []
status: experimental
availability: implemented
notice: This experimental native method is implemented in the standalone compiler and VM.
version: "0.1"
owner: std.dataStructures.TypedMap
visibility: public
modifiers: []
---

Returns a fresh [`MapIterator`](thp:std.spl.MapIterator) preserving keys without coercion. Later mutations do not change its captured entries. Object keys and values retain identity. Each `foreach` obtains a fresh cursor.

## See also

- [`TypedMap`](thp:std.dataStructures.TypedMap)
