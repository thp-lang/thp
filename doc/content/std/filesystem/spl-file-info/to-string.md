---
kind: method
id: std.spl.SplFileInfo::__toString
title: SplFileInfo::__toString
summary: Returns the pathname.
name: __toString
order: 29
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the pathname.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::__toString()` returns the pathname.

## Behavior

Returns the pathname.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->__toString();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
