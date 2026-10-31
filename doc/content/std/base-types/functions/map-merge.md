---
kind: function
id: std.baseTypes.map_merge
title: map_merge
summary: Merges two maps while preserving first key positions.
name: map_merge
order: 16
typeParameters:
  - name: K
    description: Shared key type.
  - name: V
    description: Shared value type.
parameters:
  - name: first
    type: map<K, V>
    description: Initial entries.
  - name: second
    type: map<K, V>
    description: Entries whose values replace matching initial keys.
returns:
  type: map<K, V>
  description: Combined entries in first-seen key order.
errors:
  - description: Runtime allocation failure stops the operation without a partial result.
related: []
status: experimental
availability: implemented
notice: This experimental contract executes in the standalone compiler and reference VM.
version: "0.6"
module: base-types
---

`map_merge()` visits the first map, then the second. A key found in both maps
keeps its first position and takes the second map's value. New second-map keys
append in their order. Neither input changes. An empty literal infers `K` and
`V` from the other operand; two empty literals need an expected `map<K, V>`
result type. No callback runs. Allocation failure stops without returning a
partial map.

```thp
$merged = map_merge({"a" => 1, "b" => 2}, {"b" => 9, "c" => 3});
```
