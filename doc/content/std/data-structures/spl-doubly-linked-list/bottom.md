---
kind: method
id: std.spl.SplDoublyLinkedList::bottom
title: SplDoublyLinkedList::bottom
summary: Returns the value at the beginning.
name: bottom
order: 8
typeParameters: []
parameters: []
returns:
  type: T
  description: Returns the value at the beginning.
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::bottom()` returns the value at the beginning.

## Behavior

Returns the value at the beginning.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->bottom();
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
