---
kind: method
id: std.spl.SplPriorityQueue::extract
title: SplPriorityQueue::extract
summary: Removes and returns the highest-priority value.
name: extract
order: 5
typeParameters: []
parameters: []
returns:
  type: T
  description: Removes and returns the highest-priority value.
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

[`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)`::extract()` removes and returns the highest-priority value.

## Behavior

Removes and returns the highest-priority value.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->extract();
```

The call uses the signature and defaults documented above.

## See also

- [`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)
