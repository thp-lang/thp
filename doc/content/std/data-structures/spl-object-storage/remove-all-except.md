---
kind: method
id: std.spl.SplObjectStorage::removeAllExcept
title: SplObjectStorage::removeAllExcept
summary: Retains only identities present in $storage and returns the number removed.
name: removeAllExcept
order: 7
typeParameters: []
parameters:
  - name: storage
    type: SplObjectStorage<TInfo>
    description: Value supplied as $storage.
returns:
  type: int
  description: Retains only identities present in $storage and returns the number removed.
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

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::removeAllExcept()` retains only identities present in $storage and returns the number removed.

## Behavior

Retains only identities present in $storage and returns the number removed.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->removeAllExcept($storage);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
