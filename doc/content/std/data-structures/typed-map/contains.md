---
kind: method
id: std.dataStructures.TypedMap::contains
title: TypedMap::contains
summary: "Tests whether a key is present."
name: contains
order: 3
typeParameters: []
parameters:
  - name: key
    type: "K"
    description: "Key matched using native map equality without coercion."
returns:
  type: "bool"
  description: "Tests whether a key is present."
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

Returns `true` even when the associated value is null. Does not mutate the map.

## See also

- [`TypedMap`](thp:std.dataStructures.TypedMap)
