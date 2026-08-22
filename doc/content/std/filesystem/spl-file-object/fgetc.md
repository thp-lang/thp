---
kind: method
id: std.spl.SplFileObject::fgetc
title: SplFileObject::fgetc
summary: Reads one byte, or false at end of file.
name: fgetc
order: 13
typeParameters: []
parameters: []
returns:
  type: string|false
  description: Reads one byte, or false at end of file.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::fgetc()` reads one byte, or false at end of file.

## Behavior

Reads one byte, or false at end of file.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->fgetc();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
