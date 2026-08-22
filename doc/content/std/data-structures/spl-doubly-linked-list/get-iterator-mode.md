---
kind: method
id: std.spl.SplDoublyLinkedList::getIteratorMode
title: SplDoublyLinkedList::getIteratorMode
summary: Returns direction and destructive-iteration mode.
name: getIteratorMode
order: 12
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns direction and destructive-iteration mode.
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::getIteratorMode()` returns direction and destructive-iteration mode.

## Behavior

Returns direction and destructive-iteration mode.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIteratorMode();
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
