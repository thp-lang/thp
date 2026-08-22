---
kind: method
id: std.baseTypes.Iterator::rewind
title: Iterator::rewind
summary: Positions the cursor on the first element.
name: rewind
order: 1
typeParameters: []
parameters: []
returns:
  type: void
  description: This method does not return a value.
errors:
  - description: Failures encountered while restarting the underlying source propagate.
related: []
status: experimental
availability: proposed
notice: This member belongs to an experimental API contract and is not implemented in this repository.
version: "0.1"
owner: std.baseTypes.Iterator
visibility: public
modifiers: []
---

[`Iterator`](thp:std.baseTypes.Iterator)`::rewind()` positions the cursor on
the first element.

## Behavior

After a successful call, `valid()` returns `true` when the iterator contains
an element and `false` when it is empty. Restartable iterators return to their
first element after partial or complete traversal. A one-shot implementation
may fail when called after it has advanced, but it must accept the initial
`rewind()` performed on a fresh iterator.

## Example

```thp
$iterator->rewind();

if ($iterator->valid()) {
    echo $iterator->value();
}
```

## See also

- [`Iterator`](thp:std.baseTypes.Iterator)
- [`valid()`](thp:std.baseTypes.Iterator::valid)
