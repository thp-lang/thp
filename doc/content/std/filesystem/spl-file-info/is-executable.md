---
kind: method
id: std.spl.SplFileInfo::isExecutable
title: SplFileInfo::isExecutable
summary: Reports whether the path is executable.
name: isExecutable
order: 18
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the path is executable.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::isExecutable()` reports whether the path is executable.

## Behavior

Reports whether the path is executable.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isExecutable();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
