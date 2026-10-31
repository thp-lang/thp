---
kind: function
id: std.spl.iterator_to_map
title: iterator_to_map
summary: Consumes remaining iterator entries into an insertion-ordered map.
name: iterator_to_map
order: 9
typeParameters:
  - name: K
    description: The iterator key type.
  - name: V
    description: The iterator value type.
parameters:
  - name: iterator
    type: Iterator<K, V>
    description: Cursor to consume from its current position.
returns:
  type: map<K, V>
  description: Remaining entries in traversal order.
errors:
  - description: Cursor failures propagate unchanged.
related: []
status: experimental
availability: implemented
notice: This experimental native function is implemented in the standalone compiler and VM.
version: "0.5"
module: iterators
---

`iterator_to_map()` calls `valid()`, `key()`, `value()`, and `advance()` until
exhaustion. It does not rewind. Repeated keys replace the value at that key's
first insertion position, as native map assignment does. An exhausted iterator
produces an empty map.

```thp
$cursor = new VectorIterator([4, 5]);
$remaining: map<int, int> = iterator_to_map($cursor); // {0 => 4, 1 => 5}
```
