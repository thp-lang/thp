---
kind: method
id: std.spl.SplFileObject::fread
title: SplFileObject::fread
summary: Reads at most $length bytes, or returns false.
name: fread
order: 4
typeParameters: []
parameters:
  - name: length
    type: int
    description: Maximum number of bytes or values processed.
returns:
  type: string|false
  description: Reads at most $length bytes, or returns false.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::fread()` reads at most $length bytes, or returns false.

## Behavior

Reads at most $length bytes, or returns false.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->fread($length);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
