---
kind: method
id: std.dataStructures.TypedMap::toMap
title: TypedMap::toMap
summary: "Returns the current map value snapshot."
name: toMap
order: 6
typeParameters: []
parameters: []
returns:
  type: "map<K, V>"
  description: "Returns the current map value snapshot."
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

Preserves keys and insertion order. The result is a copy-on-write native map value: later writes to it or mutations through the wrapper do not alter each other. Objects in keys or values retain identity; this is not a deep copy.

## See also

- [`TypedMap`](thp:std.dataStructures.TypedMap)
