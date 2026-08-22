---
kind: method
id: std.spl.SplObjectStorage::offsetExists
title: SplObjectStorage::offsetExists
summary: Reports whether an offset exists.
name: offsetExists
order: 9
typeParameters: []
parameters:
  - name: object
    type: object
    description: Object identity addressed by the operation.
returns:
  type: bool
  description: Reports whether an offset exists.
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

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::offsetExists()` reports whether an offset exists.

## Behavior

Reports whether an offset exists.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetExists($object);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
