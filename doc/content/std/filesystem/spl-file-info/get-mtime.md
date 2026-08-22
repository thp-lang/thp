---
kind: method
id: std.spl.SplFileInfo::getMTime
title: SplFileInfo::getMTime
summary: Returns the modification timestamp, or false when unavailable.
name: getMTime
order: 13
typeParameters: []
parameters: []
returns:
  type: int|false
  description: Returns the modification timestamp, or false when unavailable.
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
owner: std.spl.SplFileInfo
visibility: public
modifiers: []
---

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::getMTime()` returns the modification timestamp, or false when unavailable.

## Behavior

Returns the modification timestamp, or false when unavailable.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getMTime();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
