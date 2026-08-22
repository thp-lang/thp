---
kind: method
id: std.spl.SplObjectStorage::addAll
title: SplObjectStorage::addAll
summary: Adds every object-information entry and returns the number added.
name: addAll
order: 5
typeParameters: []
parameters:
  - name: storage
    type: SplObjectStorage<TInfo>
    description: Value supplied as $storage.
returns:
  type: int
  description: Adds every object-information entry and returns the number added.
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

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::addAll()` adds every object-information entry and returns the number added.

## Behavior

Adds every object-information entry and returns the number added.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->addAll($storage);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
