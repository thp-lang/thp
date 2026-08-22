---
kind: method
id: std.spl.SplDoublyLinkedList::getIterator
title: SplDoublyLinkedList::getIterator
summary: Returns an iterator using the selected direction.
name: getIterator
order: 18
typeParameters: []
parameters: []
returns:
  type: Iterator<int, T>
  description: Returns an iterator using the selected direction.
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::getIterator()` returns an iterator using the selected direction.

## Behavior

Returns an iterator using the selected direction.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
