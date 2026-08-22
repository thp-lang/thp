---
kind: method
id: std.spl.ArrayObject::getArrayCopy
title: ArrayObject::getArrayCopy
summary: Returns a map copy of the stored entries.
name: getArrayCopy
order: 7
typeParameters: []
parameters: []
returns:
  type: map<K, V>
  description: Returns a map copy of the stored entries.
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

[`ArrayObject`](thp:std.spl.ArrayObject)`::getArrayCopy()` returns a map copy of the stored entries.

## Behavior

Returns a map copy of the stored entries.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getArrayCopy();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
