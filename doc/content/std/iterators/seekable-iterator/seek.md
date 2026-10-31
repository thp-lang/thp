---
kind: method
id: std.spl.SeekableIterator::seek
title: SeekableIterator::seek
summary: Moves the iterator to a requested position.
name: seek
order: 1
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
      Throws OutOfBoundsException when the requested position is negative or
      unavailable. Failures from the underlying iterator propagate.
related: []
status: experimental
availability: implemented
notice: LimitIterator implements this cursor method.
version: "0.1"
owner: std.spl.SeekableIterator
visibility: public
modifiers: []
---

[`SeekableIterator`](thp:std.spl.SeekableIterator)`::seek()` moves the iterator to a requested position.

## Behavior

Moves the iterator to a requested position.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->seek($offset);
```

The call uses the signature and defaults documented above.

## See also

- [`SeekableIterator`](thp:std.spl.SeekableIterator)
