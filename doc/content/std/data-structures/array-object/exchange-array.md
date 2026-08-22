---
kind: method
id: std.spl.ArrayObject::exchangeArray
title: ArrayObject::exchangeArray
summary: Replaces storage and returns the previous map.
name: exchangeArray
order: 18
typeParameters: []
parameters:
  - name: values
    type: map<K, V>|object
    description: Initial values consumed or stored by the operation.
returns:
  type: map<K, V>
  description: Replaces storage and returns the previous map.
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

[`ArrayObject`](thp:std.spl.ArrayObject)`::exchangeArray()` replaces storage and returns the previous map.

## Behavior

Replaces storage and returns the previous map.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->exchangeArray($values);
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
