---
kind: method
id: std.baseTypes.IteratorAggregate::getIterator
title: IteratorAggregate::getIterator
summary: Returns a fresh typed iterator for one traversal.
name: getIterator
order: 1
typeParameters: []
parameters: []
returns:
  type: Iterator<K, V>
  description: A fresh iterator whose cursor state is independent of earlier traversals.
errors:
  - description: Failures encountered while creating the iterator propagate.
related: []
status: experimental
availability: proposed
notice: This member belongs to an experimental API contract and is not implemented in this repository.
version: "0.1"
owner: std.baseTypes.IteratorAggregate
visibility: public
modifiers: []
---

[`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate)`::getIterator()`
returns a fresh typed iterator for one traversal.

## Behavior

The returned `Iterator<K, V>` has cursor state independent of iterators returned
by earlier calls. `foreach` calls `rewind()` on that iterator before reading its
first element.

## Example

```thp
$iterator = $values->getIterator();
$iterator->rewind();
```

## See also

- [`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate)
- [`Iterator`](thp:std.baseTypes.Iterator)
