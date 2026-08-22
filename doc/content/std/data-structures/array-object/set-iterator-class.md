---
kind: method
id: std.spl.ArrayObject::setIteratorClass
title: ArrayObject::setIteratorClass
summary: Selects the keyed iterator class returned for traversal.
name: setIteratorClass
order: 19
typeParameters: []
parameters:
  - name: iterator_class
    type: string
    description: Qualified name of an ArrayIterator-compatible class.
returns:
  type: void
  description: This method does not return a value.
errors:
  - description:
      The call fails before changing state when the named class does not exist,
      cannot be constructed for this storage, or is not compatible with ArrayIterator.
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

[`ArrayObject`](thp:std.spl.ArrayObject)`::setIteratorClass()` selects the keyed iterator class returned for traversal.

## Behavior

Selects the keyed iterator class returned for traversal.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setIteratorClass($iterator_class);
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
