---
kind: method
id: std.spl.DirectoryIterator::seek
title: DirectoryIterator::seek
summary: Moves the iterator to a requested position.
name: seek
order: 6
typeParameters: []
parameters:
  - name: offset
    type: int
    description: Position addressed by the operation.
returns:
  type: void
  description: This method does not return a value.
errors:
  - description:
      The operation fails when the requested position is invalid or the
      underlying source cannot seek.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.DirectoryIterator
visibility: public
modifiers: []
---

[`DirectoryIterator`](thp:std.spl.DirectoryIterator)`::seek()` moves the iterator to a requested position.

## Behavior

Moves the iterator to a requested position.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->seek($offset);
```

The call uses the signature and defaults documented above.

## See also

- [`DirectoryIterator`](thp:std.spl.DirectoryIterator)
