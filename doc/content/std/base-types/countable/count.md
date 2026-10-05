---
kind: method
id: std.baseTypes.Countable::count
title: Countable::count
summary: Returns the number of represented items.
name: count
order: 1
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the number of represented items.
errors:
  - description: The VM throws UnexpectedValueException if the result is negative.
related: []
status: experimental
availability: implemented
notice: This method executes in the reference VM.
version: "0.1"
owner: std.baseTypes.Countable
visibility: public
modifiers: []
---

[`Countable`](thp:std.baseTypes.Countable)`::count()` returns the number of represented items.

## Behavior

Returns the number of represented items.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->count();
```

The call uses the signature and defaults documented above.

## See also

- [`Countable`](thp:std.baseTypes.Countable)
