---
kind: method
id: std.spl.SplObjectStorage::offsetUnset
title: SplObjectStorage::offsetUnset
summary: Removes the value at an offset.
name: offsetUnset
order: 12
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

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::offsetUnset()` removes the value at an offset.

## Behavior

Removes the value at an offset.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetUnset($object);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
