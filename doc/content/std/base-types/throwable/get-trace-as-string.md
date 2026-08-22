---
kind: method
id: std.baseTypes.Throwable::getTraceAsString
title: Throwable::getTraceAsString
summary: Returns the stack trace as text.
name: getTraceAsString
order: 6
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the stack trace as text.
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

[`Throwable`](thp:std.baseTypes.Throwable)`::getTraceAsString()` returns the stack trace as text.

## Behavior

Returns the stack trace as text.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getTraceAsString();
```

The call uses the signature and defaults documented above.

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)
