---
kind: method
id: std.spl.SplDoublyLinkedList::shift
title: SplDoublyLinkedList::shift
summary: Removes and returns the first value.
name: shift
order: 4
typeParameters: []
parameters: []
returns:
  type: T
  description: Removes and returns the first value.
errors:
  - description:
      The operation fails when the container is empty. Comparison or delegated
      runtime failures propagate where applicable.
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::shift()` removes and returns the first value.

## Behavior

Removes and returns the first value.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->shift();
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
