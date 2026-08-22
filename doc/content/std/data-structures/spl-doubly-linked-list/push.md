---
kind: method
id: std.spl.SplDoublyLinkedList::push
title: SplDoublyLinkedList::push
summary: Appends a value.
name: push
order: 5
typeParameters: []
parameters:
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::push()` appends a value.

## Behavior

Appends a value.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->push($value);
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
