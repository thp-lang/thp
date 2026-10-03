---
kind: function
id: std.spl.iterator_to_vector
title: iterator_to_vector
summary: Consumes remaining iterator values into a vector.
name: iterator_to_vector
order: 8
typeParameters:
  - name: K
    description: The ignored iterator key type.
  - name: V
    description: The iterator value type.
parameters:
  - name: iterator
    type: Iterator<K, V>
    description: Cursor to consume from its current position.
returns:
  type: vector<V>
  description: Remaining values in traversal order.
errors:
  - description: Cursor failures propagate unchanged.
related: []
status: experimental
availability: implemented
notice: This experimental native function is implemented in the standalone compiler and VM.
version: "0.5"
module: iterators
---

`iterator_to_vector()` calls `valid()`, `value()`, and `advance()` until
exhaustion. It does not rewind. Keys are discarded. An already exhausted
iterator produces an empty vector.

```thp
$cursor = new MapIterator({"a" => 1, "b" => 2});
$cursor->advance();
$remaining: vector<int> = iterator_to_vector($cursor); // [2]
```
