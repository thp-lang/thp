---
kind: method
id: std.spl.SplFileObject::fstat
title: SplFileObject::fstat
summary: Returns file metadata.
name: fstat
order: 17
typeParameters: []
parameters: []
returns:
  type: map<string, int>
  description: Returns file metadata.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::fstat()` returns file metadata.

## Behavior

Returns file metadata.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->fstat();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
