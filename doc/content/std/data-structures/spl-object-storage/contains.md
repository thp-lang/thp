---
kind: method
id: std.spl.SplObjectStorage::contains
title: SplObjectStorage::contains
summary: Reports whether an object is stored.
name: contains
order: 4
typeParameters: []
parameters:
  - name: object
    type: object
    description: Object identity addressed by the operation.
returns:
  type: bool
  description: Reports whether an object is stored.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
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

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::contains()` reports whether an object is stored.

## Behavior

Reports whether an object is stored.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->contains($object);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
