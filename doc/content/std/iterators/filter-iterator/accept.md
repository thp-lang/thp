---
kind: method
id: std.spl.FilterIterator::accept
title: FilterIterator::accept
summary: Reports whether the current value passes the filter.
name: accept
order: 2
typeParameters: []
parameters:
  - name: value
    type: V
    description: Value consumed or stored by the operation.
returns:
  type: bool
  description: Reports whether the current value passes the filter.
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
owner: std.spl.FilterIterator
visibility: public
modifiers:
  - abstract
---

[`FilterIterator`](thp:std.spl.FilterIterator)`::accept()` reports whether the current value passes the filter.

## Behavior

Reports whether the current value passes the filter.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->accept($value);
```

The call uses the signature and defaults documented above.

## See also

- [`FilterIterator`](thp:std.spl.FilterIterator)
