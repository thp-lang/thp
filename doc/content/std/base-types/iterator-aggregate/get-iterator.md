---
kind: method
id: std.baseTypes.IteratorAggregate::getIterator
title: IteratorAggregate::getIterator
summary: Returns the next typed traversable layer for one traversal.
name: getIterator
order: 1
typeParameters: []
parameters: []
returns:
  type: Traversable<K, V>
  description: The next aggregate layer or direct iterator for this traversal.
errors:
  - description: Failures encountered while creating the iterator propagate.
related: []
status: experimental
availability: implemented
notice: This method signature participates in executable interface checks and virtual dispatch.
version: "0.1"
owner: std.baseTypes.IteratorAggregate
visibility: public
modifiers: []
---

[`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate)`::getIterator()`
returns the next typed traversable layer for one traversal.

## Behavior

`foreach` calls this method exactly once for the current aggregate layer. A
returned `Iterator<K, V>` receives one initial `rewind()`; a returned
`IteratorAggregate<K, V>` receives the same one-call dispatch at its layer.
Failures propagate unchanged. Implementations should normally return fresh
cursor state so separate traversals do not interfere.

## Example

```thp
$traversable = $values->getIterator();
```

## See also

- [`IteratorAggregate`](thp:std.baseTypes.IteratorAggregate)
- [`Iterator`](thp:std.baseTypes.Iterator)
