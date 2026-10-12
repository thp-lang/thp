---
kind: method
id: std.spl.SplFileInfo::isLink
title: SplFileInfo::isLink
summary: Reports whether the path is a symbolic link.
name: isLink
order: 21
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the path is a symbolic link.
errors:
  - description:
      Underlying I/O failures follow the return sentinel shown in the signature
      or propagate as the experimental THP I/O failure where no sentinel is available.
related: []
status: experimental
availability: implemented
notice: This method executes against the current host filesystem.
version: "0.1"
owner: std.spl.SplFileInfo
visibility: public
modifiers: []
---

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::isLink()` reports whether the path is a symbolic link.

## Behavior

Reports whether the path is a symbolic link.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isLink();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
