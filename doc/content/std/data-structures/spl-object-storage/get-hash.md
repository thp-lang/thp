---
kind: method
id: std.spl.SplObjectStorage::getHash
title: SplObjectStorage::getHash
summary: Returns the identity hash used for an object.
name: getHash
order: 13
typeParameters: []
parameters:
  - name: object
    type: object
    description: Object identity addressed by the operation.
returns:
  type: string
  description: Returns the identity hash used for an object.
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

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::getHash()` returns the identity hash used for an object.

## Behavior

Returns the identity hash used for an object.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getHash($object);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
