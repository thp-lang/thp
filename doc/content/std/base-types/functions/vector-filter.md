---
kind: function
id: std.baseTypes.vector_filter
title: vector_filter
summary: Retains vector values accepted by a typed callback.
name: vector_filter
order: 11
typeParameters:
  - name: T
    description: Element type.
parameters:
  - name: values
    type: vector<T>
    description: Input values in sequence order.
  - name: callback
    type: callable<T, bool>
    description: Receives one value and decides whether to retain it.
returns:
  type: vector<T>
  description: Dense vector of retained values.
errors:
  - description: Callback exceptions and runtime allocation failures propagate without a partial result.
related: []
status: experimental
availability: implemented
notice: This experimental contract executes in the standalone compiler and reference VM.
version: "0.6"
module: base-types
---

`vector_filter()` calls the callback once per value and keeps only `true`
results. The callback receives `T`, never an index. Retained values are
renumbered from zero. The input is unchanged. An empty literal takes `T` from
the callback parameter; no callback runs. Callback exceptions and allocation
failures stop the operation without returning a partial vector.

```thp
$even = vector_filter([1, 2, 3], fn(int $value): bool => $value % 2 == 0);
```
