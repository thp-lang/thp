---
kind: method
id: std.baseTypes.Serializable::serialize
title: Serializable::serialize
summary: Returns the object's serialized payload.
name: serialize
order: 1
typeParameters: []
parameters: []
returns:
  type: ?string
  description: Returns the object's serialized payload.
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
owner: std.baseTypes.Serializable
visibility: public
modifiers: []
---

[`Serializable`](thp:std.baseTypes.Serializable)`::serialize()` returns the object's serialized payload.

## Behavior

Returns the object's serialized payload.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->serialize();
```

The call uses the signature and defaults documented above.

## See also

- [`Serializable`](thp:std.baseTypes.Serializable)
