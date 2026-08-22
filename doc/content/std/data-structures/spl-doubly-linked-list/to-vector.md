---
kind: method
id: std.spl.SplDoublyLinkedList::toVector
title: SplDoublyLinkedList::toVector
summary: Returns values in index order.
name: toVector
order: 17
typeParameters: []
parameters: []
returns:
  type: vector<T>
  description: Returns values in index order.
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::toVector()` returns values in index order.

## Behavior

Returns values in index order.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->toVector();
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
