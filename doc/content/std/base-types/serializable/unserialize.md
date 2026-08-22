---
kind: method
id: std.baseTypes.Serializable::unserialize
title: Serializable::unserialize
summary: Restores object state from a payload.
name: unserialize
order: 2
typeParameters: []
parameters:
  - name: data
    type: string
    description: Data processed by this operation.
returns:
  type: void
  description: This method does not return a value.
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

[`Serializable`](thp:std.baseTypes.Serializable)`::unserialize()` restores object state from a payload.

## Behavior

Restores object state from a payload.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->unserialize($data);
```

The call uses the signature and defaults documented above.

## See also

- [`Serializable`](thp:std.baseTypes.Serializable)
