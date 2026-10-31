---
kind: class
id: std.spl.RecursiveFilterIterator
title: RecursiveFilterIterator
summary: Defines filtering for recursive iterators.
name: RecursiveFilterIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: T
    description: The T type parameter.
parent:
  id: std.spl.FilterIterator
  arguments:
    - K
    - RecursiveEntry<K, T>
interfaces:
  - id: std.spl.RecursiveIterator
    arguments:
      - K
      - T
  - id: std.spl.OuterIterator
    arguments:
      - K
      - RecursiveEntry<K, T>
constants: []
properties: []
status: experimental
availability: implemented
notice:
  This experimental abstract class is implemented. Concrete subclasses supply
  a boolean accept method for an entry and its key.
version: "0.1"
---

This is an abstract class.

`RecursiveFilterIterator` defines filtering for recursive iterators.

## Construction

| Method                                                              | Description                                                                                 |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RecursiveFilterIterator::__construct) | Wraps a recursive iterator. Subclasses decide which complete recursive entries are yielded. |

## Behavior

Concrete subclasses implement `accept(RecursiveEntry<K, T>, K): bool`.
Accepted entries retain their values; child iterators are wrapped with a copy
of the same filter policy. Callback failures propagate to the caller.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
function visit<K, T>(RecursiveFilterIterator<K, T> $values): void {
    foreach ($values as $value) {
        print($value->value());
    }
}
```

Concrete subclasses decide which values remain visible.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RecursiveFilterIterator`](https://www.php.net/manual/en/class.recursivefilteriterator.php)
