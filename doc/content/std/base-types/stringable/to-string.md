---
kind: method
id: std.baseTypes.Stringable::__toString
title: Stringable::__toString
summary: Returns the object's string form.
name: __toString
order: 1
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the object's string form.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
notice: This method executes in the reference VM.
version: "0.1"
owner: std.baseTypes.Stringable
visibility: public
modifiers: []
---

[`Stringable`](thp:std.baseTypes.Stringable)`::__toString()` returns the object's string form.

## Behavior

Returns the object's string form.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->__toString();
```

The call uses the signature and defaults documented above.

## See also

- [`Stringable`](thp:std.baseTypes.Stringable)
