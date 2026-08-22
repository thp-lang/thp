---
kind: method
id: std.spl.SplDoublyLinkedList::pop
title: SplDoublyLinkedList::pop
summary: Removes and returns the final value.
name: pop
order: 3
typeParameters: []
parameters: []
returns:
  type: T
  description: Removes and returns the final value.
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::pop()` removes and returns the final value.

## Behavior

Removes and returns the final value.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->pop();
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
