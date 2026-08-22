---
kind: method
id: std.spl.RecursiveArrayIterator::offsetSet
title: RecursiveArrayIterator::offsetSet
summary: Replaces a keyed value, or appends when the offset is null.
name: offsetSet
order: 4
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
owner: std.spl.RecursiveArrayIterator
visibility: public
modifiers: []
---

[`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)`::offsetSet()` replaces a keyed value, or appends when the offset is null.

## Behavior

Replaces a keyed value, or appends when the offset is null.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetSet($offset, $value);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)
