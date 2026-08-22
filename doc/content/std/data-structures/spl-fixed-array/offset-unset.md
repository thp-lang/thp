---
kind: method
id: std.spl.SplFixedArray::offsetUnset
title: SplFixedArray::offsetUnset
summary: Resets the slot to null.
name: offsetUnset
order: 11
typeParameters: []
parameters:
  - name: index
    type: int
    description: Zero-based index addressed by the operation.
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
owner: std.spl.SplFixedArray
visibility: public
modifiers: []
---

[`SplFixedArray`](thp:std.spl.SplFixedArray)`::offsetUnset()` resets the slot to null.

## Behavior

Resets the slot to null.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetUnset($index);
```

The call uses the signature and defaults documented above.

## See also

- [`SplFixedArray`](thp:std.spl.SplFixedArray)
