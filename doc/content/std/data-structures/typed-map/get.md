---
kind: method
id: std.dataStructures.TypedMap::get
title: TypedMap::get
summary: "Reads a value or returns null for a missing key."
name: get
order: 2
typeParameters: []
parameters:
  - name: key
    type: "K"
    description: "Key matched using native map equality without coercion."
returns:
  type: "?V"
  description: "Reads a value or returns null for a missing key."
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

Does not insert a missing key. If `V` allows null, use [`contains()`](thp:std.dataStructures.TypedMap::contains) to distinguish a stored null from absence.

## See also

- [`TypedMap`](thp:std.dataStructures.TypedMap)
