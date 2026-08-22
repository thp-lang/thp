---
kind: method
id: std.spl.ArrayIterator::ksort
title: ArrayIterator::ksort
summary: Sorts entries by key.
name: ksort
order: 12
typeParameters: []
parameters:
  - name: flags
    type: int
    description: Bit mask selecting the documented options.
    default: SORT_REGULAR
returns:
  type: "true"
  description: Returns true after the operation completes.
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
owner: std.spl.ArrayIterator
visibility: public
modifiers: []
---

[`ArrayIterator`](thp:std.spl.ArrayIterator)`::ksort()` sorts entries by key.

## Behavior

Sorts entries by key.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->ksort();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayIterator`](thp:std.spl.ArrayIterator)
