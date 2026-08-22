---
kind: method
id: std.baseTypes.Throwable::__toString
title: Throwable::__toString
summary: Returns a textual representation.
name: __toString
order: 9
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns a textual representation.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: partial
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.baseTypes.Throwable
visibility: public
modifiers: []
---

[`Throwable`](thp:std.baseTypes.Throwable)`::__toString()` returns a textual representation.

## Behavior

Returns a textual representation.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->__toString();
```

The call uses the signature and defaults documented above.

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)
