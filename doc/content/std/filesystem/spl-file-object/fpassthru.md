---
kind: method
id: std.spl.SplFileObject::fpassthru
title: SplFileObject::fpassthru
summary: Writes remaining bytes to output and returns the count.
name: fpassthru
order: 14
typeParameters: []
parameters: []
returns:
  type: int
  description: Writes remaining bytes to output and returns the count.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::fpassthru()` writes remaining bytes to output and returns the count.

## Behavior

Writes remaining bytes to output and returns the count.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->fpassthru();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
