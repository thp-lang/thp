---
kind: function
id: std.baseTypes.vector_slice
title: vector_slice
summary: Copies a contiguous vector range.
name: vector_slice
order: 12
typeParameters:
  - name: T
    description: Element type.
parameters:
  - name: values
    type: vector<T>
    description: Source vector.
  - name: offset
    type: int
    description: Start offset; negative values count back from the end.
  - name: length
    type: ?int
    description: Maximum count; negative values exclude that many trailing elements.
    default: "null"
returns:
  type: vector<T>
  description: Dense copy of the selected range.
errors:
  - description: Runtime allocation failure stops the operation without a partial result.
related: []
status: experimental
availability: implemented
notice: This experimental contract executes in the standalone compiler and reference VM.
version: "0.6"
module: base-types
---

The offset is clamped to the vector bounds. A negative offset starts that many
elements before the end. A negative length ends that many elements before the
end; an omitted or `null` length runs to the end. An end before the start
produces an empty vector. Result offsets are dense from zero and the input is
unchanged. `vector_slice([], ...)` needs an expected `vector<T>` type. No
callback runs. Allocation failure stops without returning a partial vector.

```thp
$middle = vector_slice([1, 2, 3, 4], -3, -1); // [2, 3]
```
