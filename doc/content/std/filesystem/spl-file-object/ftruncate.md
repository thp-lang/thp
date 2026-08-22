---
kind: method
id: std.spl.SplFileObject::ftruncate
title: SplFileObject::ftruncate
summary: Truncates the file to $size bytes.
name: ftruncate
order: 18
typeParameters: []
parameters:
  - name: size
    type: int
    description: Requested container or file size.
returns:
  type: bool
  description: Truncates the file to $size bytes.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::ftruncate()` truncates the file to $size bytes.

## Behavior

Truncates the file to $size bytes.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->ftruncate($size);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
