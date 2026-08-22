---
kind: method
id: std.spl.MultipleIterator::detachIterator
title: MultipleIterator::detachIterator
summary: Removes an attached iterator.
name: detachIterator
order: 4
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<int, T>
    description: Iterator wrapped or consumed by this operation.
returns:
  type: void
  description: This method does not return a value.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.MultipleIterator
visibility: public
modifiers: []
---

[`MultipleIterator`](thp:std.spl.MultipleIterator)`::detachIterator()` removes an attached iterator.

## Behavior

Removes an attached iterator.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->detachIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`MultipleIterator`](thp:std.spl.MultipleIterator)
