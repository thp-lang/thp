---
kind: method
id: std.spl.SplFileObject::ftell
title: SplFileObject::ftell
summary: Returns the byte offset, or false on failure.
name: ftell
order: 11
typeParameters: []
parameters: []
returns:
  type: int|false
  description: Returns the byte offset, or false on failure.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::ftell()` returns the byte offset, or false on failure.

## Behavior

Returns the byte offset, or false on failure.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->ftell();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
