---
kind: class
id: std.dataStructures.FixedSequence
title: FixedSequence
summary: Indexed nullable slots with an explicit, resizable length.
name: FixedSequence
module: data-structures
typeParameters:
  - name: T
    description: The non-null value type of initialized slots.
interfaces:
  - id: std.baseTypes.IteratorAggregate
    arguments:
      - int
      - ?T
  - id: std.baseTypes.MapAccess
    arguments:
      - int
      - ?T
  - id: std.baseTypes.Countable
constants: []
properties: []
status: experimental
availability: implemented
notice:
  This experimental native sequence is implemented. Slots and iterator snapshots
  carry `?T`; assigning null clears an initialized slot.
version: "0.1"
---

`FixedSequence<T>` provides explicitly sized, resizable nullable slots. Choose
it when clearing a value must preserve its position and the length must remain
explicit. Prefer a native `vector<T>` when those slot semantics are unnecessary.
This wrapper does not imply a speed or memory advantage over vectors.

[`__construct()`](thp:std.dataStructures.FixedSequence::__construct) creates
zero-based slots filled with `null` (default size zero).
[`setSize()`](thp:std.dataStructures.FixedSequence::setSize) grows with null slots
or discards slots beyond the new length. Negative sizes throw `ValueError`.
[`count()`](thp:std.dataStructures.FixedSequence::count) and
[`getSize()`](thp:std.dataStructures.FixedSequence::getSize) count all slots,
including null ones.

[`offsetSet()`](thp:std.dataStructures.FixedSequence::offsetSet) writes or clears
an existing slot; a null offset throws `ValueError` rather than appending.
[`offsetUnset()`](thp:std.dataStructures.FixedSequence::offsetUnset) clears to
null without shifting positions. These operations and
[`offsetGet()`](thp:std.dataStructures.FixedSequence::offsetGet) throw
`OutOfBoundsException` for negative or out-of-range indices.
[`offsetExists()`](thp:std.dataStructures.FixedSequence::offsetExists) instead
returns false for invalid indices or null slots.

[`getIterator()`](thp:std.dataStructures.FixedSequence::getIterator) creates a
fresh `VectorIterator` snapshot of every slot, including null slots. Later
mutation does not change the captured slots. `foreach` uses `IteratorAggregate`.
Specify `T` explicitly when context cannot infer it. Use the offset methods
directly; object bracket access is not implemented.

```thp
<?thp
$slots = new FixedSequence<string>(3);
$slots->offsetSet(0, "left");
$slots->offsetSet(2, "right");
$slots->offsetUnset(0);
echo $slots->getSize(); // 3: clearing preserves positions.
echo $slots->offsetGet(2); // right
```

## Current implementation costs

These are implementation details, not API guarantees. Storage is a native
vector of nullable values plus an object wrapper. Indexed reads are constant
time. Every write and clear copies the full slot sequence, even without an
iterator snapshot. Resizing allocates a new vector and copies retained slots;
the current resize path also copies the old slot sequence first. Copies retain
object identities rather than cloning objects. Capturing an iterator adds a
cursor object sharing the current vector snapshot.

Comparative speed or memory remains unmeasured. Any claim against native vectors
requires reproducible measurements of equivalent sizing, clearing, copying,
and traversal workloads.
