---
kind: method
id: std.spl.SplPriorityQueue::isCorrupted
title: SplPriorityQueue::isCorrupted
summary: Reports whether ordering must be rebuilt.
name: isCorrupted
order: 9
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether ordering must be rebuilt.
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

[`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)`::isCorrupted()` reports whether ordering must be rebuilt.

## Behavior

Reports whether ordering must be rebuilt.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isCorrupted();
```

The call uses the signature and defaults documented above.

## See also

- [`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)
