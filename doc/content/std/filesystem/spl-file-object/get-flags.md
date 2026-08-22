---
kind: method
id: std.spl.SplFileObject::getFlags
title: SplFileObject::getFlags
summary: Returns line-iteration flags.
name: getFlags
order: 20
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns line-iteration flags.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::getFlags()` returns line-iteration flags.

## Behavior

Returns line-iteration flags.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getFlags();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
