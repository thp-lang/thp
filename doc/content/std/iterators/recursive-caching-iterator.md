---
kind: class
id: std.spl.RecursiveCachingIterator
title: RecursiveCachingIterator
summary: Combines caching and lookahead with recursive traversal.
name: RecursiveCachingIterator
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
  - id: std.baseTypes.Countable
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`RecursiveCachingIterator` combines caching and lookahead with recursive traversal.

## Construction

| Method                                                               | Description                                                                             |
| -------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RecursiveCachingIterator::__construct) | Wraps a recursive cursor iterator and optionally retains every visited recursive entry. |

## Behavior

Each yielded entry retains its original child iterator. Caches belong to one
wrapper and are not shared automatically with child iterators.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$cachedTree = new RecursiveCachingIterator<int, Node>($tree);
```

The wrapper adds recursive child access while retaining caching behavior.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RecursiveCachingIterator`](https://www.php.net/manual/en/class.recursivecachingiterator.php)
