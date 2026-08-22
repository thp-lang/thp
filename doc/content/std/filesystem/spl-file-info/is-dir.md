---
kind: method
id: std.spl.SplFileInfo::isDir
title: SplFileInfo::isDir
summary: Reports whether the path is a directory.
name: isDir
order: 20
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the path is a directory.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::isDir()` reports whether the path is a directory.

## Behavior

Reports whether the path is a directory.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isDir();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
