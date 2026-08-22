---
kind: method
id: std.spl.ArrayObject::getIterator
title: ArrayObject::getIterator
summary: Returns a fresh keyed cursor iterator.
name: getIterator
order: 17
typeParameters: []
parameters: []
returns:
  type: Iterator<K, V>
  description: Returns a fresh keyed cursor iterator.
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

[`ArrayObject`](thp:std.spl.ArrayObject)`::getIterator()` returns a fresh keyed cursor iterator.

## Behavior

Returns a fresh keyed cursor iterator.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
