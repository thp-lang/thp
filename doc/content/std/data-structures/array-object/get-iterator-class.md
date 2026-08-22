---
kind: method
id: std.spl.ArrayObject::getIteratorClass
title: ArrayObject::getIteratorClass
summary: Returns the selected keyed iterator class name.
name: getIteratorClass
order: 20
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the selected keyed iterator class name.
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

[`ArrayObject`](thp:std.spl.ArrayObject)`::getIteratorClass()` returns the selected keyed iterator class name.

## Behavior

Returns the selected keyed iterator class name.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIteratorClass();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
