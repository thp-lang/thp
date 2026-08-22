---
kind: method
id: std.baseTypes.Throwable::getMessage
title: Throwable::getMessage
summary: Returns the error message.
name: getMessage
order: 1
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the error message.
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

[`Throwable`](thp:std.baseTypes.Throwable)`::getMessage()` returns the error message.

## Behavior

Returns the error message.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = (new Exception("failed"))->getMessage();
```

The call uses the signature and defaults documented above.

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)
