---
kind: method
id: std.spl.SplPriorityQueue::recoverFromCorruption
title: SplPriorityQueue::recoverFromCorruption
summary: Rebuilds ordering after a failed comparison.
name: recoverFromCorruption
order: 8
typeParameters: []
parameters: []
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
owner: std.spl.SplPriorityQueue
visibility: public
modifiers: []
---

[`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)`::recoverFromCorruption()` rebuilds ordering after a failed comparison.

## Behavior

Rebuilds ordering after a failed comparison.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->recoverFromCorruption();
```

The call uses the signature and defaults documented above.

## See also

- [`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)
