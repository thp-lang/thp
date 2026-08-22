---
kind: method
id: std.spl.SplFileInfo::getPath
title: SplFileInfo::getPath
summary: Returns the containing directory path.
name: getPath
order: 2
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the containing directory path.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::getPath()` returns the containing directory path.

## Behavior

Returns the containing directory path.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getPath();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
