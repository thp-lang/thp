---
kind: function
id: std.baseTypes.map_transform
title: map_transform
summary: Transforms map values while preserving keys.
name: map_transform
order: 14
typeParameters:
  - name: K
    description: Key type.
  - name: V
    description: Input value type.
  - name: U
    description: Callback result type.
parameters:
  - name: values
    type: map<K, V>
    description: Input entries in insertion order.
  - name: callback
    type: callable<V, K, U>
    description: Receives value then key and returns a replacement value.
returns:
  type: map<K, U>
  description: Transformed values at their original keys and positions.
errors:
  - description: Callback exceptions and runtime allocation failures propagate without a partial result.
related: []
status: experimental
availability: implemented
notice: This experimental contract executes in the standalone compiler and reference VM.
version: "0.6"
module: base-types
---

`map_transform()` invokes the callback once per entry in insertion order,
passing `V` then `K`. Keys and their positions are unchanged. The input is
unchanged. An empty literal infers `V` and `K` from the callback parameters
and `U` from its return type; no callback runs. Callback exceptions and
allocation failures stop without returning a partial map.

```thp
$raised = map_transform({"a" => 1}, fn(int $value, string $key): int => $value + 1);
```
