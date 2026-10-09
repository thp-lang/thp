---
kind: class
id: std.dataStructures.MinHeap
title: MinHeap
summary: Extracts the least comparable value first.
name: MinHeap
module: data-structures
typeParameters:
  - name: T
    description: Comparable value type.
interfaces:
  - id: std.baseTypes.IteratorAggregate
    arguments:
      - int
      - T
  - id: std.baseTypes.Countable
constants: []
properties: []
status: experimental
availability: implemented
notice:
  This experimental native heap is implemented. Ranking currently scans the
  stored values, so top and extraction take linear time.
version: "0.1"
---

`MinHeap<T>` accepts values with `insert(T): void`. `top(): T` reads the least
value; `extract(): T` removes it. Both throw `UnderflowException` when empty.
Values must support THP's less-than comparison; incomparable values fail at
insertion, and NaN throws `ValueError`. `count()` and `isEmpty()` inspect the
current size.

`getIterator(): Traversable<int, T>` returns a least-first snapshot without
consuming the heap.

```thp
$heap = new MinHeap<int>();
$heap->insert(3);
$heap->insert(8);
echo $heap->extract();
```
