---
kind: method
id: std.baseTypes.Option::isSome
title: Option::isSome
summary: Returns true when the option was created with some().
name: isSome
order: 3
typeParameters: []
parameters: []
returns:
  type: bool
  description: Returns true when the option was created with some().
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
modifiers: []
---

[`Option`](thp:std.baseTypes.Option)`::isSome()` returns true when the option was created with some().

## Behavior

Returns true when the option was created with some().

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isSome();
```

The call uses the signature and defaults documented above.

## See also

- [`Option`](thp:std.baseTypes.Option)
