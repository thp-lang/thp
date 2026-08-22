---
kind: method
id: std.spl.SplFileObject::__toString
title: SplFileObject::__toString
summary: Returns the current line.
name: __toString
order: 25
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the current line.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::__toString()` returns the current line.

## Behavior

Returns the current line.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->__toString();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
