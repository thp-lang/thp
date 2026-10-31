---
kind: method
id: std.spl.SplFileInfo::getRealPath
title: SplFileInfo::getRealPath
summary: Returns the canonical path, or false on failure.
name: getRealPath
order: 23
typeParameters: []
parameters: []
returns:
  type: string|false
  description: Returns the canonical path, or false on failure.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::getRealPath()` returns the canonical path, or false on failure.

## Behavior

Returns the canonical path, or false on failure.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getRealPath();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
