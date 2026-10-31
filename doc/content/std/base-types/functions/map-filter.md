---
kind: function
id: std.baseTypes.map_filter
title: map_filter
summary: Retains map entries accepted by a typed callback.
name: map_filter
order: 15
typeParameters:
  - name: K
    description: Key type.
  - name: V
    description: Value type.
parameters:
  - name: values
    type: map<K, V>
    description: Input entries in insertion order.
  - name: callback
    type: callable<V, K, bool>
    description: Receives value then key and decides whether to retain the entry.
returns:
  type: map<K, V>
  description: Retained entries with their original keys and order.
errors:
  - description: Callback exceptions and runtime allocation failures propagate without a partial result.
related: []
status: experimental
availability: implemented
notice: This experimental contract executes in the standalone compiler and reference VM.
version: "0.6"
module: base-types
---

`map_filter()` calls the callback once per entry in insertion order, passing
`V` then `K`. Only `true` retains an entry. Retained keys keep their original
positions relative to one another; the input is unchanged. An empty literal
infers `V` and `K` from the callback parameters; no callback runs. Callback
exceptions and allocation failures stop without returning a partial map.

```thp
$large = map_filter({"a" => 1, "b" => 3}, fn(int $value, string $key): bool => $value > 2);
```
