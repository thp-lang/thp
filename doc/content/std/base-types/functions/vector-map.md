---
kind: function
id: std.baseTypes.vector_map
title: vector_map
summary: Transforms each vector value with a typed callback.
name: vector_map
order: 10
typeParameters:
  - name: T
    description: Input element type.
  - name: U
    description: Callback result type.
parameters:
  - name: values
    type: vector<T>
    description: Input values in sequence order.
  - name: callback
    type: callable<T, U>
    description: Receives one value and returns its replacement.
returns:
  type: vector<U>
  description: Dense transformed vector.
errors:
  - description: Callback exceptions and runtime allocation failures propagate without a partial result.
related: []
status: experimental
availability: implemented
notice: This experimental contract executes in the standalone compiler and reference VM.
version: "0.6"
module: base-types
---

`vector_map()` calls the callback once for each value, in order. The callback
receives `T`, never an index. Result offsets are dense from zero. The input is
unchanged. On an empty literal, `T` comes from the callback parameter and `U`
from its declared return type; no callback runs. Callback exceptions and
allocation failures stop the operation without returning a partial vector.

```thp
$doubled = vector_map([1, 2], fn(int $value): int => $value * 2);
```
