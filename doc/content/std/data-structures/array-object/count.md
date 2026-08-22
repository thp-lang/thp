---
kind: method
id: std.spl.ArrayObject::count
title: ArrayObject::count
summary: Returns the number of represented values.
name: count
order: 8
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the number of represented values.
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
owner: std.spl.ArrayObject
visibility: public
modifiers: []
---

[`ArrayObject`](thp:std.spl.ArrayObject)`::count()` returns the number of represented values.

## Behavior

Returns the number of represented values.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->count();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
