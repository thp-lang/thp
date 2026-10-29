---
kind: method
id: std.dataStructures.TypedMap::set
title: TypedMap::set
summary: "Inserts or replaces a keyed value."
name: set
order: 1
typeParameters: []
parameters:
  - name: key
    type: "K"
    description: "Key matched using native map equality without coercion."
  - name: value
    type: "V"
    description: "Value to associate with the key."
returns:
  type: "void"
  description: "This method does not return a value."
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

New keys append in insertion order. Replacing a key preserves its position. Object aliases observe the change; existing iterator and map value snapshots retain their previous entries.

## See also

- [`TypedMap`](thp:std.dataStructures.TypedMap)
