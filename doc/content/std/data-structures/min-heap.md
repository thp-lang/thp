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

`MinHeap<T>` accepts values with [`insert(T): void`](thp:std.dataStructures.MinHeap::insert). [`top(): T`](thp:std.dataStructures.MinHeap::top) reads the least
value; [`extract(): T`](thp:std.dataStructures.MinHeap::extract) removes it. Both throw `UnderflowException` when empty.
Values must support THP's less-than comparison; incomparable values fail at
insertion, and NaN throws `ValueError`. [`count()`](thp:std.dataStructures.MinHeap::count) and [`isEmpty()`](thp:std.dataStructures.MinHeap::isEmpty) inspect the
current size.

[`getIterator(): Traversable<int, T>`](thp:std.dataStructures.MinHeap::getIterator) returns a least-first snapshot without
consuming the heap.

```thp
<?thp
$heap = new MinHeap<int>();
$heap->insert(3);
$heap->insert(8);
echo $heap->extract();
```

Current implementation detail: an object wrapper stores values in a vector.
Insertion copies that sequence, top and extraction scan it, and extraction
also copies remaining values. Ordered iterator snapshots take quadratic time.
Use a native vector when ordered removal is unnecessary. These costs are not
API guarantees or measured comparative performance claims.
