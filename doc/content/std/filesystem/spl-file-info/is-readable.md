---
kind: method
id: std.spl.SplFileInfo::isReadable
title: SplFileInfo::isReadable
summary: Reports whether the path is readable.
name: isReadable
order: 17
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the path is readable.
errors:
  - description:
      Underlying I/O failures follow the return sentinel shown in the signature
      or propagate as the experimental THP I/O failure where no sentinel is available.
related: []
status: experimental
availability: partial
notice: This method uses basic host metadata; access-control lists and effective-user checks are not implemented.
version: "0.1"
owner: std.spl.SplFileInfo
visibility: public
modifiers: []
---

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::isReadable()` reports whether the path is readable.

## Behavior

Reports whether the path is readable.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isReadable();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
