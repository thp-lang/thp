---
kind: method
id: std.baseTypes.MapAccess::offsetSet
title: MapAccess::offsetSet
summary:
  null requests an implementation-defined append key. Implementations that do not
  support append reject null.
name: offsetSet
order: 3
typeParameters: []
parameters:
  - name: offset
    type: K|null
    description: Position addressed by the operation.
  - name: value
    type: V
    description: Value consumed or stored by the operation.
returns:
  type: void
  description: This method does not return a value.
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
owner: std.baseTypes.MapAccess
visibility: public
modifiers: []
---

[`MapAccess`](thp:std.baseTypes.MapAccess)`::offsetSet()` null requests an implementation-defined append key. Implementations that do not support append reject null.

## Behavior

null requests an implementation-defined append key. Implementations that do not support append reject null.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetSet($offset, $value);
```

The call uses the signature and defaults documented above.

## See also

- [`MapAccess`](thp:std.baseTypes.MapAccess)
