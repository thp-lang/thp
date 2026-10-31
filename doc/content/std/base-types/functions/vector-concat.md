---
kind: function
id: std.baseTypes.vector_concat
title: vector_concat
summary: Appends one vector's values after another.
name: vector_concat
order: 13
typeParameters:
  - name: T
    description: Shared element type.
parameters:
  - name: first
    type: vector<T>
    description: Leading values.
  - name: second
    type: vector<T>
    description: Trailing values.
returns:
  type: vector<T>
  description: Dense vector in left-then-right order.
errors:
  - description: Runtime allocation failure stops the operation without a partial result.
related: []
status: experimental
availability: implemented
notice: This experimental contract executes in the standalone compiler and reference VM.
version: "0.6"
module: base-types
---

`vector_concat()` reads each input in order and returns dense offsets from
zero. Neither input changes. An empty literal infers `T` from the other
operand; two empty literals need an expected `vector<T>` result type. No
callback runs. Allocation failure stops without returning a partial vector.

```thp
$all = vector_concat([1, 2], [3, 4]);
```
