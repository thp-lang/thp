---
kind: method
id: std.spl.MultipleIterator::attachIterator
title: MultipleIterator::attachIterator
summary: Adds an iterator to the lockstep group.
name: attachIterator
order: 3
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

[`MultipleIterator`](thp:std.spl.MultipleIterator)`::attachIterator()` adds an iterator to the lockstep group.

## Behavior

Adds an iterator to the lockstep group.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->attachIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`MultipleIterator`](thp:std.spl.MultipleIterator)
