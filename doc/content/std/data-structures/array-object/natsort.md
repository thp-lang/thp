---
kind: method
id: std.spl.ArrayObject::natsort
title: ArrayObject::natsort
summary: Sorts values using natural ordering.
name: natsort
order: 15
typeParameters: []
parameters: []
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

[`ArrayObject`](thp:std.spl.ArrayObject)`::natsort()` sorts values using natural ordering.

## Behavior

Sorts values using natural ordering.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->natsort();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
