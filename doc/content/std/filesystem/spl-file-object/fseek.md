---
kind: method
id: std.spl.SplFileObject::fseek
title: SplFileObject::fseek
summary: Moves to $offset relative to $whence.
name: fseek
order: 12
typeParameters: []
parameters:
  - name: offset
    type: int
    description: Position addressed by the operation.
  - name: whence
    type: int
    description: Origin used to interpret the offset.
    default: SEEK_SET
returns:
  type: int
  description: Moves to $offset relative to $whence.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::fseek()` moves to $offset relative to $whence.

## Behavior

Moves to $offset relative to $whence.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->fseek($offset);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
