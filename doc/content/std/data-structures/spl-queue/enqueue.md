---
kind: method
id: std.spl.SplQueue::enqueue
title: SplQueue::enqueue
summary: Adds a value to the back of the queue.
name: enqueue
order: 2
typeParameters: []
parameters:
  - name: value
    type: T
    description: Value consumed or stored by the operation.
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
owner: std.spl.SplQueue
visibility: public
modifiers: []
---

[`SplQueue`](thp:std.spl.SplQueue)`::enqueue()` adds a value to the back of the queue.

## Behavior

Adds a value to the back of the queue.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->enqueue($value);
```

The call uses the signature and defaults documented above.

## See also

- [`SplQueue`](thp:std.spl.SplQueue)
