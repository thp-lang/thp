---
kind: method
id: std.spl.SplObjectStorage::offsetSet
title: SplObjectStorage::offsetSet
summary: Stores object information; a null offset is rejected.
name: offsetSet
order: 11
typeParameters: []
parameters:
  - name: object
    type: object|null
    description: Object identity addressed by the operation.
  - name: info
    type: ?TInfo
    description: Information associated with the object.
    default: "null"
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
owner: std.spl.SplObjectStorage
visibility: public
modifiers: []
---

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::offsetSet()` stores object information; a null offset is rejected.

## Behavior

Stores object information; a null offset is rejected.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetSet($object);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
