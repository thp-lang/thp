---
kind: method
id: std.dataStructures.TypedMap::remove
title: TypedMap::remove
summary: "Removes a keyed entry if present."
name: remove
order: 4
typeParameters: []
parameters:
  - name: key
    type: "K"
    description: "Key matched using native map equality without coercion."
returns:
  type: "bool"
  description: "Removes a keyed entry if present."
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

Returns `true` when an entry was removed and `false` when the key was absent. Other entries retain their order. Reinserting a removed key appends it. Object aliases observe removal; existing snapshots keep their entries.

## See also

- [`TypedMap`](thp:std.dataStructures.TypedMap)
