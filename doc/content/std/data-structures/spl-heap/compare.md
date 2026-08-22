---
kind: method
id: std.spl.SplHeap::compare
title: SplHeap::compare
summary: Compares two heap values.
name: compare
order: 1
typeParameters: []
parameters:
  - name: value1
    type: T
    description: Left value.
  - name: value2
    type: T
    description: Right value.
returns:
  type: int
  description:
    A positive integer when $value1 ranks above $value2, zero when they rank
    equally, or a negative integer when it ranks below.
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
owner: std.spl.SplHeap
visibility: protected
modifiers:
  - abstract
---

[`SplHeap`](thp:std.spl.SplHeap)`::compare()` compares two heap values.

## Behavior

Compares two heap values.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

Use a subclass to customize ordering:

```thp
final class RankedHeap extends SplHeap<int>
{

    protected function compare(int $left, int $right): int
    {
        return $left <=> $right;
    }
}
```

The protected method is invoked by heap operations; callers do not call it
directly.

## See also

- [`SplHeap`](thp:std.spl.SplHeap)
