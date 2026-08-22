---
kind: method
id: std.baseTypes.Throwable::getCode
title: Throwable::getCode
summary: Returns the error code.
name: getCode
order: 2
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the error code.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: partial
notice:
  The compiler and reference VM implement this member for Exception, Error, and
  their descendants.
version: "0.1"
owner: std.baseTypes.Throwable
visibility: public
modifiers: []
---

[`Throwable`](thp:std.baseTypes.Throwable)`::getCode()` returns the error code.

## Behavior

Returns the error code.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = (new Exception("failed", 7))->getCode();
```

The call uses the signature and defaults documented above.

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)
