---
kind: method
id: std.spl.RecursiveArrayIterator::offsetUnset
title: RecursiveArrayIterator::offsetUnset
summary: Removes the value at the key.
name: offsetUnset
order: 5
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
owner: std.spl.RecursiveArrayIterator
visibility: public
modifiers: []
---

[`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)`::offsetUnset()` removes the value at the key.

## Behavior

Removes the value at the key.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetUnset($offset);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)
