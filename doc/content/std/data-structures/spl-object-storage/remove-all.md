---
kind: method
id: std.spl.SplObjectStorage::removeAll
title: SplObjectStorage::removeAll
summary: Removes identities present in $storage and returns the number removed.
name: removeAll
order: 6
typeParameters: []
parameters:
  - name: storage
    type: SplObjectStorage<TInfo>
    description: Value supplied as $storage.
returns:
  type: int
  description: Removes identities present in $storage and returns the number removed.
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

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::removeAll()` removes identities present in $storage and returns the number removed.

## Behavior

Removes identities present in $storage and returns the number removed.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->removeAll($storage);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
