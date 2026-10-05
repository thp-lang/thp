---
kind: method
id: std.baseTypes.Option::get
title: Option::get
summary:
  Returns the value supplied to some(). Calling get() on an absent option raises
  a runtime error.
name: get
order: 5
typeParameters: []
parameters: []
returns:
  type: T
  description:
    Returns the value supplied to some(). Calling get() on an absent option
    raises a runtime error.
errors:
  - description: Throws OutOfBoundsException when the option is absent.
related: []
status: experimental
availability: implemented
notice: This method executes in the reference VM.
version: "0.1"
owner: std.baseTypes.Option
visibility: public
modifiers: []
---

[`Option`](thp:std.baseTypes.Option)`::get()` returns the value supplied to some(). Calling get() on an absent option raises a runtime error.

## Behavior

Returns the value supplied to some(). Calling get() on an absent option raises a runtime error.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->get();
```

The call uses the signature and defaults documented above.

## See also

- [`Option`](thp:std.baseTypes.Option)
