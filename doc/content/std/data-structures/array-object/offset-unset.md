---
kind: method
id: std.spl.ArrayObject::offsetUnset
title: ArrayObject::offsetUnset
summary: Removes the value at an offset.
name: offsetUnset
order: 5
typeParameters: []
parameters:
  - name: key
    type: K
    description: Key addressed by the operation.
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
owner: std.spl.ArrayObject
visibility: public
modifiers: []
---

[`ArrayObject`](thp:std.spl.ArrayObject)`::offsetUnset()` removes the value at an offset.

## Behavior

Removes the value at an offset.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetUnset($key);
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
