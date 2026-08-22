---
kind: method
id: std.spl.SplDoublyLinkedList::add
title: SplDoublyLinkedList::add
summary: Inserts a value at an index.
name: add
order: 2
typeParameters: []
parameters:
  - name: index
    type: int
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::add()` inserts a value at an index.

## Behavior

Inserts a value at an index.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->add($index, $value);
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
