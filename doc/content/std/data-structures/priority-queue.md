---
kind: class
id: std.dataStructures.PriorityQueue
title: PriorityQueue
summary: Extracts values by descending integer priority.
name: PriorityQueue
module: data-structures
typeParameters:
  - name: T
    description: Stored value type.
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
  This experimental native priority queue is implemented. Ranking currently
  scans the stored values, so top and extraction take linear time.
version: "0.1"
---

`PriorityQueue<T>` accepts [`insert(T $value, int $priority): void`](thp:std.dataStructures.PriorityQueue::insert). [`top(): T`](thp:std.dataStructures.PriorityQueue::top)
reads and [`extract(): T`](thp:std.dataStructures.PriorityQueue::extract) removes the value with greatest priority. Ties retain
insertion order. Reading or removing from an empty queue throws
`UnderflowException`. [`count()`](thp:std.dataStructures.PriorityQueue::count) and [`isEmpty()`](thp:std.dataStructures.PriorityQueue::isEmpty) inspect the current size.

[`getIterator(): Traversable<int, T>`](thp:std.dataStructures.PriorityQueue::getIterator) returns a priority-ordered snapshot
without consuming the queue.

```thp
<?thp
$queue = new PriorityQueue<string>();
$queue->insert("low", 1);
$queue->insert("high", 9);
echo $queue->extract();
```

Current implementation detail: an object wrapper stores values in a vector.
Each insertion stores a value/priority/sequence record.
Insertion copies the record sequence, top and extraction scan it, and extraction
also copies remaining values. Ordered iterator snapshots take quadratic time.
Use a native vector when ordered removal is unnecessary. These costs are not
API guarantees or measured comparative performance claims.
