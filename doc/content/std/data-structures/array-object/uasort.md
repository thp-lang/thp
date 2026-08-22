---
kind: method
id: std.spl.ArrayObject::uasort
title: ArrayObject::uasort
summary: Sorts values with $callback, preserving keys.
name: uasort
order: 13
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
owner: std.spl.ArrayObject
visibility: public
modifiers: []
---

[`ArrayObject`](thp:std.spl.ArrayObject)`::uasort()` sorts values with $callback, preserving keys.

## Behavior

Sorts values with $callback, preserving keys.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->uasort($callback);
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
