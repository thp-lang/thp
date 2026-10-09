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

`FixedSequence<T>` stores zero-based slots with a size chosen at construction.
New slots contain `null`. `setSize()` grows with null slots or discards slots
beyond the new length. `count()` and `getSize()` both return the current length.

| Method                                     | Result                                                                    |
| ------------------------------------------ | ------------------------------------------------------------------------- |
| `__construct(int $size = 0)`               | Creates the sequence; negative sizes throw `ValueError`.                  |
| `setSize(int $size): void`                 | Resizes; negative sizes throw `ValueError`.                               |
| `offsetExists(int $offset): bool`          | True for a non-null in-range slot.                                        |
| `offsetGet(int $offset): ?T`               | Reads a slot; out-of-range indices throw `OutOfBoundsException`.          |
| `offsetSet(?int $offset, ?T $value): void` | Writes an in-range slot; `null` offsets throw `ValueError`.               |
| `offsetUnset(int $offset): void`           | Clears a slot to null; out-of-range indices throw `OutOfBoundsException`. |
| `getIterator(): Traversable<int, ?T>`      | Creates a `VectorIterator` snapshot of the current slots.                 |

The generic argument is explicit when the constructor has no value from which
to infer `T`. The object uses `IteratorAggregate`, so `foreach` follows the
common iterator protocol and obtains one fresh cursor per traversal.

```thp
$slots = new FixedSequence<string>(2);
$slots->offsetSet(0, "left");
foreach ($slots as $index => $value) {
    echo $index;
}
```
