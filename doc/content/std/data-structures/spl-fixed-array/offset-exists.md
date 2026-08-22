---
kind: method
id: std.spl.SplFixedArray::offsetExists
title: SplFixedArray::offsetExists
summary: Reports whether an initialized slot exists.
name: offsetExists
order: 8
typeParameters: []
parameters:
  - name: index
    type: int
    description: Zero-based index addressed by the operation.
returns:
  type: bool
  description: Reports whether an initialized slot exists.
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

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::offsetExists()` reports whether an initialized slot exists.

## Behavior

Reports whether an initialized slot exists.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetExists($index);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
