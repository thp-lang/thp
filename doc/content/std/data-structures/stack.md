---
kind: class
id: std.dataStructures.Stack
title: Stack
summary: Last-in, first-out sequence.
name: Stack
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
notice: This experimental native stack is implemented with snapshot iteration.
version: "0.1"
---

`Stack<T>` uses inherited [`push(T)`](thp:std.dataStructures.LinkedList::push) and [`pop(): T`](thp:std.dataStructures.LinkedList::pop) for LIFO access.
[`pop()`](thp:std.dataStructures.LinkedList::pop) throws `UnderflowException` when empty. `foreach` returns a fresh
aggregate cursor from top to bottom. [`toVector()`](thp:std.dataStructures.LinkedList::toVector) retains insertion order.

```thp
<?thp
$stack = new Stack<int>();
$stack->push(1);
$stack->push(2);
echo $stack->pop();
```

This adapter uses the same vector-backed storage and copying mutations as
[`LinkedList`](thp:std.dataStructures.LinkedList); it does not provide a separate
constant-time deque implementation.
