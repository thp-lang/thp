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

`LinkedList<T>` supports [`push(T)`](thp:std.dataStructures.LinkedList::push), [`pop(): T`](thp:std.dataStructures.LinkedList::pop), [`unshift(T)`](thp:std.dataStructures.LinkedList::unshift), and [`shift(): T`](thp:std.dataStructures.LinkedList::shift).
[`top(): T`](thp:std.dataStructures.LinkedList::top) reads the last value; [`bottom(): T`](thp:std.dataStructures.LinkedList::bottom) reads the first. Reading or
removing from an empty list throws `UnderflowException`. [`count()`](thp:std.dataStructures.LinkedList::count) and
[`isEmpty()`](thp:std.dataStructures.LinkedList::isEmpty) inspect the current length. [`toVector()`](thp:std.dataStructures.LinkedList::toVector) returns a value snapshot.

[`getIterator(): Traversable<int, T>`](thp:std.dataStructures.LinkedList::getIterator) returns a fresh `VectorIterator` snapshot
in front-to-back order. Mutating the list after obtaining a cursor does not
change that cursor. `foreach` uses the normal `IteratorAggregate` protocol.

```thp
<?thp
$list = new LinkedList<int>();
$list->push(2);
$list->unshift(1);
foreach ($list as $value) {
    echo $value;
}
```

Current implementation detail: despite the name, storage is a vector behind an
object wrapper. Every end mutation copies the sequence; head operations also
shift entries. Length and end reads are constant time. Prefer a native vector
when shared mutable double-ended operations are unnecessary. These costs are
not API guarantees.
