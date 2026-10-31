---
kind: method
id: std.spl.SplFileInfo::getATime
title: SplFileInfo::getATime
summary: Returns the access timestamp, or false when unavailable.
name: getATime
order: 12
typeParameters: []
parameters: []
returns:
  type: int|false
  description: Returns the access timestamp, or false when unavailable.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::getATime()` returns the access timestamp, or false when unavailable.

## Behavior

Returns the access timestamp, or false when unavailable.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getATime();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
