---
kind: method
id: std.spl.SplFileObject::getMaxLineLen
title: SplFileObject::getMaxLineLen
summary: Returns the configured maximum line length.
name: getMaxLineLen
order: 22
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the configured maximum line length.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::getMaxLineLen()` returns the configured maximum line length.

## Behavior

Returns the configured maximum line length.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getMaxLineLen();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
