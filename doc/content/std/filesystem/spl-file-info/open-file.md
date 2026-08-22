---
kind: method
id: std.spl.SplFileInfo::openFile
title: SplFileInfo::openFile
summary: Opens the path with the requested mode and stream context.
name: openFile
order: 26
typeParameters: []
parameters:
  - name: mode
    type: string
    description: Mode selected from the values documented below.
    default: '"r"'
  - name: use_include_path
    type: bool
    description: Value supplied as $use_include_path.
    default: "false"
  - name: context
    type: mixed
    description: Optional stream context.
    default: "null"
returns:
  type: SplFileObject
  description: Opens the path with the requested mode and stream context.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::openFile()` opens the path with the requested mode and stream context.

## Behavior

Opens the path with the requested mode and stream context.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->openFile();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
