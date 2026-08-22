---
kind: method
id: std.spl.SplFileInfo::getFileInfo
title: SplFileInfo::getFileInfo
summary: Returns information for this path using the requested class.
name: getFileInfo
order: 24
typeParameters: []
parameters:
  - name: class
    type: ?string
    description: Qualified class name used by this operation.
    default: "null"
returns:
  type: SplFileInfo
  description: Returns information for this path using the requested class.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::getFileInfo()` returns information for this path using the requested class.

## Behavior

Returns information for this path using the requested class.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getFileInfo();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
