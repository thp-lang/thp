---
kind: method
id: std.spl.ArrayObject::__construct
title: ArrayObject::__construct
summary:
  Wraps the supplied map or public object properties and selects the iterator
  class used by getIterator().
name: __construct
order: 1
typeParameters: []
parameters:
  - name: values
    type: map<K, V>|object
    description: Initial values consumed or stored by the operation.
    default: "{}"
  - name: flags
    type: int
    description: Bit mask selecting the documented options.
    default: "0"
  - name: iterator_class
    type: string
    description: Qualified name of an ArrayIterator-compatible class.
    default: '"ArrayIterator"'
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Construction fails when an argument violates the documented contract or an
      underlying resource cannot be created. Concrete THP error classes remain
      experimental unless named above.
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

[`ArrayObject`](thp:std.spl.ArrayObject)`::__construct()` wraps the supplied map or public object properties and selects the iterator class used by getIterator().

## Behavior

Wraps the supplied map or public object properties and selects the iterator class used by getIterator().

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new ArrayObject();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
