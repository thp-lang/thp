---
kind: method
id: std.spl.SplFileObject::flock
title: SplFileObject::flock
summary: Attempts a lock and returns both outcome flags.
name: flock
order: 9
typeParameters: []
parameters:
  - name: operation
    type: int
    description: Lock operation assembled from the documented lock constants.
returns:
  type: FileLockResult
  description: Attempts a lock and returns both outcome flags.
errors:
  - description:
      Underlying I/O failures follow the return sentinel shown in the signature
      or propagate as the experimental THP I/O failure where no sentinel is available.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.SplFileObject
visibility: public
modifiers: []
---

[`SplFileObject`](thp:std.spl.SplFileObject)`::flock()` attempts a lock and returns both outcome flags.

## Behavior

Attempts a lock and returns both outcome flags.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->flock($operation);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
