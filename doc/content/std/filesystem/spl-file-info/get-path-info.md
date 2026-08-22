---
kind: method
id: std.spl.SplFileInfo::getPathInfo
title: SplFileInfo::getPathInfo
summary: Returns information for the containing path.
name: getPathInfo
order: 25
typeParameters: []
parameters:
  - name: class
    type: ?string
    description: Qualified class name used by this operation.
    default: "null"
returns:
  type: ?SplFileInfo
  description: Returns information for the containing path.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::getPathInfo()` returns information for the containing path.

## Behavior

Returns information for the containing path.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getPathInfo();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
