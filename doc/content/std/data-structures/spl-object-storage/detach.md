---
kind: method
id: std.spl.SplObjectStorage::detach
title: SplObjectStorage::detach
summary: Removes an object identity and its information.
name: detach
order: 3
typeParameters: []
parameters:
  - name: object
    type: object
    description: Object identity addressed by the operation.
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

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::detach()` removes an object identity and its information.

## Behavior

Removes an object identity and its information.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->detach($object);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
