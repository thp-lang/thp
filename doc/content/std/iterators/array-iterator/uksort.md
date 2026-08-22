---
kind: method
id: std.spl.ArrayIterator::uksort
title: ArrayIterator::uksort
summary: Sorts keys with $callback.
name: uksort
order: 14
typeParameters: []
parameters:
  - name: callback
    type: callable
    description: Callable invoked by this operation.
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

[`ArrayIterator`](thp:std.spl.ArrayIterator)`::uksort()` sorts keys with $callback.

## Behavior

Sorts keys with $callback.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->uksort($callback);
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayIterator`](thp:std.spl.ArrayIterator)
