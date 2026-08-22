---
kind: method
id: std.spl.CallbackFilterIterator::accept
title: CallbackFilterIterator::accept
summary: Invokes the callback for the current value.
name: accept
order: 2
typeParameters: []
parameters:
  - name: value
    type: V
    description: Value consumed or stored by the operation.
returns:
  type: bool
  description: Invokes the callback for the current value.
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
owner: std.spl.CallbackFilterIterator
visibility: public
modifiers: []
---

[`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator)`::accept()` invokes the callback for the current value.

## Behavior

Invokes the callback for the current value.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->accept($value);
```

The call uses the signature and defaults documented above.

## See also

- [`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator)
