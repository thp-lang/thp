---
kind: method
id: std.spl.SplQueue::dequeue
title: SplQueue::dequeue
summary: Removes and returns the front value.
name: dequeue
order: 3
typeParameters: []
parameters: []
returns:
  type: T
  description: Removes and returns the front value.
errors:
  - description:
      The operation fails when the container is empty. Comparison or delegated
      runtime failures propagate where applicable.
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

[`SplQueue`](thp:std.spl.SplQueue)`::dequeue()` removes and returns the front value.

## Behavior

Removes and returns the front value.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->dequeue();
```

The call uses the signature and defaults documented above.

## See also

- [`SplQueue`](thp:std.spl.SplQueue)
