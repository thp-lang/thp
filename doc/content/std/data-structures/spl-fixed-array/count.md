---
kind: method
id: std.spl.SplFixedArray::count
title: SplFixedArray::count
summary: Returns the number of allocated slots.
name: count
order: 2
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the number of allocated slots.
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
owner: std.spl.SplFixedArray
visibility: public
modifiers: []
---

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::count()` returns the number of allocated slots.

## Behavior

Returns the number of allocated slots.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->count();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
