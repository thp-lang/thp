---
kind: method
id: std.spl.SplDoublyLinkedList::offsetSet
title: SplDoublyLinkedList::offsetSet
summary: Stores a value or appends when the index is null.
name: offsetSet
order: 15
typeParameters: []
parameters:
  - name: index
    type: int|null
    description: Zero-based index addressed by the operation.
  - name: value
    type: T
    description: Value consumed or stored by the operation.
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
owner: std.spl.SplDoublyLinkedList
visibility: public
modifiers: []
---

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::offsetSet()` stores a value or appends when the index is null.

## Behavior

Stores a value or appends when the index is null.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetSet($index, $value);
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
