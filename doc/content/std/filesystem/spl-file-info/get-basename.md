---
kind: method
id: std.spl.SplFileInfo::getBasename
title: SplFileInfo::getBasename
summary: Returns the basename after removing an optional suffix.
name: getBasename
order: 5
typeParameters: []
parameters:
  - name: suffix
    type: string
    description: Optional suffix removed from the returned basename.
    default: '""'
returns:
  type: string
  description: Returns the basename after removing an optional suffix.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::getBasename()` returns the basename after removing an optional suffix.

## Behavior

Returns the basename after removing an optional suffix.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getBasename();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
