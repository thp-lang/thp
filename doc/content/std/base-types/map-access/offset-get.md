---
kind: method
id: std.baseTypes.MapAccess::offsetGet
title: MapAccess::offsetGet
summary: Returns the value at an offset.
name: offsetGet
order: 2
typeParameters: []
parameters:
  - name: offset
    type: K
    description: Position addressed by the operation.
returns:
  type: V
  description: Returns the value at an offset.
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
owner: std.baseTypes.MapAccess
visibility: public
modifiers: []
---

[`MapAccess`](thp:std.baseTypes.MapAccess)`::offsetGet()` returns the value at an offset.

## Behavior

Returns the value at an offset.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetGet($offset);
```

The call uses the signature and defaults documented above.

## See also

- [`MapAccess`](thp:std.baseTypes.MapAccess)
