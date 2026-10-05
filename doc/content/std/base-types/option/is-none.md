---
kind: method
id: std.baseTypes.Option::isNone
title: Option::isNone
summary: Returns true when the option was created with none().
name: isNone
order: 4
typeParameters: []
parameters: []
returns:
  type: bool
  description: Returns true when the option was created with none().
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
notice: This method executes in the reference VM.
version: "0.1"
owner: std.baseTypes.Option
visibility: public
modifiers: []
---

[`Option`](thp:std.baseTypes.Option)`::isNone()` returns true when the option was created with none().

## Behavior

Returns true when the option was created with none().

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isNone();
```

The call uses the signature and defaults documented above.

## See also

- [`Option`](thp:std.baseTypes.Option)
