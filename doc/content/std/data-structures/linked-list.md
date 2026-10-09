---
kind: class
id: std.dataStructures.LinkedList
title: LinkedList
summary: Mutable double-ended sequence with snapshot iteration.
name: LinkedList
module: data-structures
typeParameters:
  - name: T
    description: The element type.
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
notice: This experimental class is implemented with indexed aggregate iteration.
  Head mutations currently copy the stored sequence.
version: "0.1"
---

`LinkedList<T>` supports `push(T)`, `pop(): T`, `unshift(T)`, and `shift(): T`.
`top(): T` reads the last value; `bottom(): T` reads the first. Reading or
removing from an empty list throws `UnderflowException`. `count()` and
`isEmpty()` inspect the current length. `toVector()` returns a value snapshot.

`getIterator(): Traversable<int, T>` returns a fresh `VectorIterator` snapshot
in front-to-back order. Mutating the list after obtaining a cursor does not
change that cursor. `foreach` uses the normal `IteratorAggregate` protocol.

```thp
$list = new LinkedList<int>();
$list->push(2);
$list->unshift(1);
foreach ($list as $value) {
    echo $value;
}
```
