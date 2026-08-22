---
kind: method
id: std.spl.SplFixedArray::toVector
title: SplFixedArray::toVector
summary: Returns all slots in index order.
name: toVector
order: 3
typeParameters: []
parameters: []
returns:
  type: vector<?T>
  description: Returns all slots in index order.
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
owner: std.spl.SplFixedArray
visibility: public
modifiers: []
---

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::toVector()` returns all slots in index order.

## Behavior

Returns all slots in index order.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->toVector();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
