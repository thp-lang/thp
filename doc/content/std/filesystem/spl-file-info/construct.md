---
kind: method
id: std.spl.SplFileInfo::__construct
title: SplFileInfo::__construct
summary: Stores the supplied path without requiring that the path already exist.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: filename
    type: string
    description: Path of the file to open.
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Construction fails when an argument violates the documented contract or an
      underlying resource cannot be created. Concrete THP error classes remain
      experimental unless named above.
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

[`SplFileInfo`](thp:std.spl.SplFileInfo)`::__construct()` stores the supplied path without requiring that the path already exist.

## Behavior

Stores the supplied path without requiring that the path already exist.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new SplFileInfo($filename);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileInfo`](thp:std.spl.SplFileInfo)
