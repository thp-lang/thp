---
kind: method
id: std.spl.ArrayIterator::asort
title: ArrayIterator::asort
summary: Sorts values while preserving key associations.
name: asort
order: 11
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

[`ArrayIterator`](thp:std.spl.ArrayIterator)`::asort()` sorts values while preserving key associations.

## Behavior

Sorts values while preserving key associations.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->asort();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayIterator`](thp:std.spl.ArrayIterator)
