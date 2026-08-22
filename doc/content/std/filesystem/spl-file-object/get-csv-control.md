---
kind: method
id: std.spl.SplFileObject::getCsvControl
title: SplFileObject::getCsvControl
summary: Returns CSV delimiter, enclosure, and escape characters.
name: getCsvControl
order: 8
typeParameters: []
parameters: []
returns:
  type: vector<string>
  description: Returns CSV delimiter, enclosure, and escape characters.
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

[`SplFileObject`](thp:std.spl.SplFileObject)`::getCsvControl()` returns CSV delimiter, enclosure, and escape characters.

## Behavior

Returns CSV delimiter, enclosure, and escape characters.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getCsvControl();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
