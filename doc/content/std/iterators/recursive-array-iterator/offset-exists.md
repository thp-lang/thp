---
kind: method
id: std.spl.RecursiveArrayIterator::offsetExists
title: RecursiveArrayIterator::offsetExists
summary: Reports whether the copied collection contains the key.
name: offsetExists
order: 2
typeParameters: []
parameters:
  - name: offset
    type: K
    description: Position addressed by the operation.
returns:
  type: bool
  description: Reports whether the copied collection contains the key.
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
owner: std.spl.RecursiveArrayIterator
visibility: public
modifiers: []
---

[`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)`::offsetExists()` reports whether the copied collection contains the key.

## Behavior

Reports whether the copied collection contains the key.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetExists($offset);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)
