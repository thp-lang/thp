---
kind: method
id: std.spl.SplObjectStorage::offsetGet
title: SplObjectStorage::offsetGet
summary: Returns the value at an offset.
name: offsetGet
order: 10
typeParameters: []
parameters:
  - name: object
    type: object
    description: Object identity addressed by the operation.
returns:
  type: ?TInfo
  description: Returns the value at an offset.
errors:
  - description:
      The operation fails when the requested key or index is unavailable.
      Concrete THP error classes remain experimental.
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

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::offsetGet()` returns the value at an offset.

## Behavior

Returns the value at an offset.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetGet($object);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
