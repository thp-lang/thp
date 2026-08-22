---
kind: method
id: std.spl.SplObjectStorage::__construct
title: SplObjectStorage::__construct
summary: Creates empty identity-based object storage.
name: __construct
order: 1
typeParameters: []
parameters: []
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
owner: std.spl.SplObjectStorage
visibility: public
modifiers: []
---

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::__construct()` creates empty identity-based object storage.

## Behavior

Creates empty identity-based object storage.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new SplObjectStorage();
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
