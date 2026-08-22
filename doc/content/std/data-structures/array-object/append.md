---
kind: method
id: std.spl.ArrayObject::append
title: ArrayObject::append
summary: Appends a value using the next supported key.
name: append
order: 6
typeParameters: []
parameters:
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

[`ArrayObject`](thp:std.spl.ArrayObject)`::append()` appends a value using the next supported key.

## Behavior

Appends a value using the next supported key.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->append($value);
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
