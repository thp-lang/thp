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
interfaces:
  - id: std.spl.RecursiveIterator
    arguments:
      - K
      - T
  - id: std.spl.OuterIterator
    arguments:
      - K
      - RecursiveEntry<T>
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

This is an abstract class.

`RecursiveFilterIterator` defines filtering for recursive iterators.

## Construction

| Method                                                              | Description                                                                                 |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RecursiveFilterIterator::__construct) | Wraps a recursive iterator. Subclasses decide which complete recursive entries are yielded. |

## Behavior

Concrete subclasses implement `accept()`. Accepted entries retain their
original child iterators; recursive filtering of descendants is performed by
wrapping those child iterators with the same policy.

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
