---
kind: method
id: std.spl.SplFileObject::seek
title: SplFileObject::seek
summary: Moves the iterator to a requested position.
name: seek
order: 23
typeParameters: []
parameters:
  - name: line
    type: int
    description: Value supplied as $line.
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
owner: std.spl.SplFileObject
visibility: public
modifiers: []
---

[`SplFileObject`](thp:std.spl.SplFileObject)`::seek()` moves the iterator to a requested position.

## Behavior

Moves the iterator to a requested position.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->seek($line);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
