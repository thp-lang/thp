---
kind: method
id: std.spl.RecursiveArrayIterator::__construct
title: RecursiveArrayIterator::__construct
summary: Copies the supplied map or public object properties for traversal.
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
owner: std.spl.RecursiveArrayIterator
visibility: public
modifiers: []
---

[`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)`::__construct()` copies the supplied map or public object properties for traversal.

## Behavior

Copies the supplied map or public object properties. The iterator exposes keys
through `key()` and values through `RecursiveEntry<V>` objects.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new RecursiveArrayIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)
