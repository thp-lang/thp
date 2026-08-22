---
kind: method
id: std.spl.SplDoublyLinkedList::offsetGet
title: SplDoublyLinkedList::offsetGet
summary: Returns the value at an offset.
name: offsetGet
order: 14
typeParameters: []
parameters:
  - name: index
    type: int
    description: Zero-based index addressed by the operation.
returns:
  type: T
  description: Returns the value at an offset.
errors:
  - description:
      The operation fails when the requested key or index is unavailable.
      Concrete THP error classes remain experimental.
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::offsetGet()` returns the value at an offset.

## Behavior

Returns the value at an offset.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetGet($index);
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
