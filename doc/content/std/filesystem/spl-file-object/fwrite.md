---
kind: method
id: std.spl.SplFileObject::fwrite
title: SplFileObject::fwrite
summary: Writes data and returns bytes written, or false.
name: fwrite
order: 16
typeParameters: []
parameters:
  - name: data
    type: string
    description: Data processed by this operation.
  - name: length
    type: ?int
    description: Maximum number of bytes or values processed.
    default: "null"
returns:
  type: int|false
  description: Writes data and returns bytes written, or false.
errors:
  - description:
      Underlying I/O failures follow the return sentinel shown in the signature
      or propagate as the experimental THP I/O failure where no sentinel is available.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::fwrite()` writes data and returns bytes written, or false.

## Behavior

Writes data and returns bytes written, or false.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->fwrite($data);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
