---
kind: method
id: std.spl.SplDoublyLinkedList::top
title: SplDoublyLinkedList::top
summary: Returns the final value without removing it.
name: top
order: 7
typeParameters: []
parameters: []
returns:
  type: T
  description: Returns the final value without removing it.
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::top()` returns the next value without removing it.

## Behavior

Returns the final value without removing it.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->top();
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
