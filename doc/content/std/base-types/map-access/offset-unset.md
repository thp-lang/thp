---
kind: method
id: std.baseTypes.MapAccess::offsetUnset
title: MapAccess::offsetUnset
summary: Removes the value at an offset.
name: offsetUnset
order: 4
typeParameters: []
parameters:
  - name: offset
    type: K
    description: Position addressed by the operation.
returns:
  type: void
  description: This method does not return a value.
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

[`MapAccess`](thp:std.baseTypes.MapAccess)`::offsetUnset()` removes the value at an offset.

## Behavior

Removes the value at an offset.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetUnset($offset);
```

The call uses the signature and defaults documented above.

## See also

- [`MapAccess`](thp:std.baseTypes.MapAccess)
