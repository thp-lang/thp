---
kind: class
id: std.spl.VectorIterator
title: VectorIterator
summary: Rewindable cursor over a vector snapshot.
name: VectorIterator
module: iterators
typeParameters:
  - name: T
    description: The vector element type.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - int
      - T
constants: []
properties: []
status: experimental
availability: implemented
notice: This experimental native iterator is implemented in the standalone compiler and VM.
version: "0.5"
---

`VectorIterator<T>` captures a copy-on-write snapshot of a `vector<T>` when
constructed. Its keys are zero-based `int` offsets. Mutating the original vector
after construction does not change the snapshot.

[`__construct()`](thp:std.spl.VectorIterator::__construct) accepts the vector.
`rewind()` moves to offset zero, `valid()` checks the current offset, and
`advance()` moves toward exhaustion. `key()` and `value()` fail on an exhausted
cursor. The cursor starts at offset zero.

```thp
$source: vector<int> = [10, 20];
$cursor = new VectorIterator($source);
$source[0] = 99;
echo $cursor->value(); // 10
```

A vector converts to a fresh `VectorIterator<T>` when a typed assignment,
property, argument, or return requires compatible `Iterator<int, T>` or
`Traversable<int, T>`. A raw vector has no cursor methods. Direct native
`foreach` keeps its collection traversal path.
