---
kind: method
id: std.spl.SplPriorityQueue::getIterator
title: SplPriorityQueue::getIterator
summary: Returns values in priority order without mutating the queue.
name: getIterator
order: 10
typeParameters: []
parameters: []
returns:
  type: Iterator<int, T>
  description: Returns values in priority order without mutating the queue.
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

[`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)`::getIterator()` returns values in priority order without mutating the queue.

## Behavior

Returns values in priority order without mutating the queue.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)
