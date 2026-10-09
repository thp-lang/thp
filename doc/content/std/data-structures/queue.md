---
kind: class
id: std.dataStructures.Queue
title: Queue
summary: First-in, first-out sequence.
name: Queue
module: data-structures
typeParameters:
  - name: T
    description: The element type.
parent:
  id: std.dataStructures.LinkedList
  arguments:
    - T
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This experimental native queue is implemented. Removing from the front
  currently copies the remaining sequence.
version: "0.1"
---

`Queue<T>` inherits the double-ended operations of `LinkedList<T>` and adds
`enqueue(T): void` and `dequeue(): T`. Enqueue appends; dequeue removes the
oldest value and throws `UnderflowException` when empty. `foreach` yields a
snapshot in FIFO order through `IteratorAggregate<int, T>`.

```thp
$queue = new Queue<string>();
$queue->enqueue("first");
$queue->enqueue("second");
echo $queue->dequeue();
```
