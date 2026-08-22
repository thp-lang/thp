---
kind: method
id: std.spl.SplDoublyLinkedList::setIteratorMode
title: SplDoublyLinkedList::setIteratorMode
summary: Sets direction and destructive-iteration mode, returning the previous mode.
name: setIteratorMode
order: 11
typeParameters: []
parameters:
  - name: mode
    type: int
    description: One direction option combined with one retention option.
returns:
  type: int
  description: The previously active combined mode.
errors:
  - description:
      The call fails before changing state when $mode does not contain exactly
      one direction option and exactly one retention option.
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

[`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)`::setIteratorMode()` sets direction and destructive-iteration mode, returning the previous mode.

## Behavior

The selected mode applies to iterators created after the change. Direction and
retention are independent; changing one preserves the bit selected for the
other only when it is included in `$mode`.

## Example

```thp
$previous = $list->setIteratorMode(
    SplDoublyLinkedList::IT_MODE_LIFO |
    SplDoublyLinkedList::IT_MODE_DELETE,
);
```

The call uses the signature and defaults documented above.

## See also

- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
