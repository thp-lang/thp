---
kind: method
id: std.baseTypes.Option::some
title: Option::some
summary: Returns an option containing $value.
name: some
order: 1
typeParameters: []
parameters:
  - name: value
    type: T
    description: Value consumed or stored by the operation.
returns:
  type: Option<T>
  description: Returns an option containing $value.
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
owner: std.baseTypes.Option
visibility: public
modifiers:
  - static
---

[`Option`](thp:std.baseTypes.Option)`::some()` returns an option containing $value.

## Behavior

Returns an option containing $value.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = Option::some($value);
```

The call uses the signature and defaults documented above.

## See also

- [`Option`](thp:std.baseTypes.Option)
