---
kind: method
id: std.baseTypes.MapAccess::offsetExists
title: MapAccess::offsetExists
summary: Reports whether an offset is present.
name: offsetExists
order: 1
typeParameters: []
parameters:
  - name: offset
    type: K
    description: Position addressed by the operation.
returns:
  type: bool
  description: Reports whether an offset is present.
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

[`MapAccess`](thp:std.baseTypes.MapAccess)`::offsetExists()` reports whether an offset is present.

## Behavior

Reports whether an offset is present.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetExists($offset);
```

The call uses the signature and defaults documented above.

## See also

- [`MapAccess`](thp:std.baseTypes.MapAccess)
