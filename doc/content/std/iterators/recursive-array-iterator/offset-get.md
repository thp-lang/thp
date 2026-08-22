---
kind: method
id: std.spl.RecursiveArrayIterator::offsetGet
title: RecursiveArrayIterator::offsetGet
summary: Returns the value for the key.
name: offsetGet
order: 3
typeParameters: []
parameters:
  - name: offset
    type: K
    description: Position addressed by the operation.
returns:
  type: V
  description: Returns the value for the key.
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

[`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)`::offsetGet()` returns the value for the key.

## Behavior

Returns the value for the key.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetGet($offset);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)
