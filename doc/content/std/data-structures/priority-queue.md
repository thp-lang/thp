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

`PriorityQueue<T>` accepts `insert(T $value, int $priority): void`. `top(): T`
reads and `extract(): T` removes the value with greatest priority. Ties retain
insertion order. Reading or removing from an empty queue throws
`UnderflowException`. `count()` and `isEmpty()` inspect the current size.

`getIterator(): Traversable<int, T>` returns a priority-ordered snapshot
without consuming the queue.

```thp
$queue = new PriorityQueue<string>();
$queue->insert("low", 1);
$queue->insert("high", 9);
echo $queue->extract();
```
