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

`Stack<T>` uses inherited `push(T)` and `pop(): T` for LIFO access.
`pop()` throws `UnderflowException` when empty. `foreach` returns a fresh
aggregate cursor from top to bottom. `toVector()` retains insertion order.

```thp
$stack = new Stack<int>();
$stack->push(1);
$stack->push(2);
echo $stack->pop();
```
