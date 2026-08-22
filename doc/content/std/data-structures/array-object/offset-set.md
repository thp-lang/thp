---
kind: method
id: std.spl.ArrayObject::offsetSet
title: ArrayObject::offsetSet
summary: Stores a value or appends when the key is null.
name: offsetSet
order: 4
typeParameters: []
parameters:
  - name: key
    type: K|null
    description: Key addressed by the operation.
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
owner: std.spl.ArrayObject
visibility: public
modifiers: []
---

[`ArrayObject`](thp:std.spl.ArrayObject)`::offsetSet()` stores a value or appends when the key is null.

## Behavior

Stores a value or appends when the key is null.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->offsetSet($key, $value);
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
