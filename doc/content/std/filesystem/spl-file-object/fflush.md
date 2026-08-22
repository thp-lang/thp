---
kind: method
id: std.spl.SplFileObject::fflush
title: SplFileObject::fflush
summary: Flushes buffered output.
name: fflush
order: 10
typeParameters: []
parameters: []
returns:
  type: bool
  description: Flushes buffered output.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::fflush()` flushes buffered output.

## Behavior

Flushes buffered output.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->fflush();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
