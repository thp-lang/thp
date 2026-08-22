---
kind: class
id: std.spl.RecursiveArrayIterator
title: RecursiveArrayIterator
summary: Adds child traversal to array-backed iteration.
name: RecursiveArrayIterator
module: iterators
typeParameters:
  - name: K
    description: The K type parameter.
  - name: V
    description: The V type parameter.
interfaces:
  - id: std.spl.RecursiveIterator
    arguments:
      - K
      - V
  - id: std.baseTypes.MapAccess
    arguments:
      - K
      - V
  - id: std.baseTypes.Countable
constants:
  - name: CHILD_ARRAYS_ONLY
    type: int
    description: Treats only map or vector values as children.
properties: []
status: experimental
availability: proposed
notice:
  This PHP migration-analysis placeholder is not an accepted THP-native API and
  is not implemented. A native replacement must distinguish vector and map
  children explicitly.
version: "0.1"
---

`RecursiveArrayIterator` records PHP migration behavior and is not a THP-native
name. THP's replacement is provisionally called
`RecursiveCollectionIterator` and must preserve whether each child is a
`vector<T>` or `map<K, V>`.

## Construction

| Method                                                             | Description                                                        |
| ------------------------------------------------------------------ | ------------------------------------------------------------------ |
| [`__construct()`](thp:std.spl.RecursiveArrayIterator::__construct) | Copies the supplied map or public object properties for traversal. |

## Behavior

The iterator cursor exposes each source key as `K`; its `RecursiveEntry<V>`
value carries the source value and optional child iterator. Map or vector
values become the `children()` iterator. With `CHILD_ARRAYS_ONLY`, objects are
not treated as child collections.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$tree = new RecursiveArrayIterator<string, mixed>({"team" => ["Ada", "Lin"]});
```

The nested team list is available as a child iterator.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RecursiveArrayIterator`](https://www.php.net/manual/en/class.recursivearrayiterator.php)
