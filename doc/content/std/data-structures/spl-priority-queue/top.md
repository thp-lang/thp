---
kind: method
id: std.spl.SplPriorityQueue::top
title: SplPriorityQueue::top
summary: Returns the highest-priority value without removing it.
name: top
order: 4
typeParameters: []
parameters: []
returns:
  type: T
  description: Returns the highest-priority value without removing it.
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
owner: std.spl.SplPriorityQueue
visibility: public
modifiers: []
---

[`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)`::top()` returns the highest-priority value without removing it.

## Behavior

Returns the highest-priority value without removing it.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->top();
```

The call uses the signature and defaults documented above.

## See also

- [`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)
