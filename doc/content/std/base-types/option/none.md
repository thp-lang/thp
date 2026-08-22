---
kind: method
id: std.baseTypes.Option::none
title: Option::none
summary: Returns an option representing absence.
name: none
order: 2
typeParameters: []
parameters: []
returns:
  type: Option<T>
  description: Returns an option representing absence.
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

[`Option`](thp:std.baseTypes.Option)`::none()` returns an option representing absence.

## Behavior

Returns an option representing absence.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = Option::none();
```

The call uses the signature and defaults documented above.

## See also

- [`Option`](thp:std.baseTypes.Option)
