---
kind: method
id: std.spl.SplPriorityQueue::compare
title: SplPriorityQueue::compare
summary: Compares two priorities.
name: compare
order: 2
typeParameters: []
parameters:
  - name: priority1
    type: P
    description: Left priority.
  - name: priority2
    type: P
    description: Right priority.
returns:
  type: int
  description:
    A positive integer when $priority1 ranks above $priority2, zero when they
    rank equally, or a negative integer when it ranks below.
errors:
  - description:
      Failures thrown by the callback or comparison operation propagate without
      being wrapped.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.SplPriorityQueue
visibility: protected
modifiers: []
---

[`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)`::compare()` compares two priorities.

## Behavior

Compares two priorities.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

Use a subclass to customize ordering:

```thp
final class RankedQueue<T> extends SplPriorityQueue<T, int>
{

    protected function compare(int $left, int $right): int
    {
        return $left <=> $right;
    }
}
```

The protected method is invoked by queue operations; callers do not call it
directly.

## See also

- [`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)
