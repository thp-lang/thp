---
kind: method
id: std.spl.SplFileInfo::getFilename
title: SplFileInfo::getFilename
summary: Returns the final path component.
name: getFilename
order: 3
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the final path component.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::getFilename()` returns the final path component.

## Behavior

Returns the final path component.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getFilename();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
