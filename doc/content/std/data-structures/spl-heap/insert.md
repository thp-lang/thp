---
kind: method
id: std.spl.SplHeap::insert
title: SplHeap::insert
summary: Adds a value to the container.
name: insert
order: 3
typeParameters: []
parameters:
  - name: value
    type: T
    description: Value consumed or stored by the operation.
returns:
  type: "true"
  description: Returns true after the operation completes.
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
owner: std.spl.SplHeap
visibility: public
modifiers: []
---

[`SplHeap`](thp:std.spl.SplHeap)`::insert()` adds a value to the container.

## Behavior

Adds a value to the container.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->insert($value);
```

The call uses the signature and defaults documented above.

## See also

- [`SplHeap`](thp:std.spl.SplHeap)
