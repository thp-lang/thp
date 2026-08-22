---
kind: method
id: std.spl.SplFixedArray::offsetGet
title: SplFixedArray::offsetGet
summary: Returns the value at an index.
name: offsetGet
order: 9
typeParameters: []
parameters:
  - name: index
    type: int
    description: Zero-based index addressed by the operation.
returns:
  type: ?T
  description: Returns the value at an index.
errors:
  - description:
      The operation fails when the requested key or index is unavailable.
      Concrete THP error classes remain experimental.
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

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::offsetGet()` returns the value at an index.

## Behavior

Returns the value at an index.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetGet($index);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
