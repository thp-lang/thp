---
kind: interface
id: std.baseTypes.Iterator
title: Iterator
summary: Traverses typed keys and values through an explicit cursor protocol.
name: Iterator
module: base-types
typeParameters:
  - name: K
    description: The type of each current key.
  - name: V
    description: The type of each current value.
interfaces:
  - id: std.baseTypes.Traversable
constants: []
properties: []
status: experimental
availability: proposed
notice: This THP-native contract is proposed and is not yet implemented in this repository.
version: "0.1"
---

`Iterator<K, V>` exposes a cursor over typed keys and values. It
uses the familiar PHP iterator state model with explicit method names and does
not allocate an entry or option object for each element.

## Methods

| Method                                             | Description                                       |
| -------------------------------------------------- | ------------------------------------------------- |
| [`rewind()`](thp:std.baseTypes.Iterator::rewind)   | Positions the cursor on the first element.        |
| [`valid()`](thp:std.baseTypes.Iterator::valid)     | Reports whether the cursor identifies an element. |
| [`key()`](thp:std.baseTypes.Iterator::key)         | Returns the current typed key.                    |
| [`value()`](thp:std.baseTypes.Iterator::value)     | Returns the current typed value.                  |
| [`advance()`](thp:std.baseTypes.Iterator::advance) | Moves the cursor to the next element.             |

## Contract

On a fresh iterator, `rewind()` positions the cursor on the first element, or
on the exhausted state when the iterator is empty. While `valid()` is true,
`key()` and `value()` return the current pair without moving the cursor.
`advance()` moves to the following pair. Once advancement reaches exhaustion,
`valid()` remains false until `rewind()` succeeds.

Calling `key()` or `value()` while `valid()` is false fails. Implementations
may check their cursor state directly; the contract does not require either
accessor to invoke the public `valid()` method.

Every iterator exposes `rewind()` so `foreach` uses one uniform protocol and
does not perform a rewindability capability check. Restartable iterators must
return to their first element on later calls. A one-shot implementation may
fail when `rewind()` is called after it has advanced; consequently a second
`foreach` over the same consumed one-shot iterator may fail. An aggregate
avoids that reuse by returning a fresh iterator for each traversal.

## Collection keys

A vector iterator implements `Iterator<int, T>` and uses zero-based offsets as
keys. A map iterator implements `Iterator<K, V>` and preserves the map's keys
and insertion order. There is no separate keyed-iterator interface.

## Example

```thp
function printEntries<K, V>(Iterator<K, V> $iterator): void {
    $iterator->rewind();

    while ($iterator->valid()) {
        echo $iterator->key() . "=" . $iterator->value() . "\n";
        $iterator->advance();
    }
}
```

## See also

- [`Traversable`](thp:std.baseTypes.Traversable)
- [`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate)
- [Iterators](thp:std.iterators)
