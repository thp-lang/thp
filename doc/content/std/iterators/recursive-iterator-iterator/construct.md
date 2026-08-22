---
kind: method
id: std.spl.RecursiveIteratorIterator::__construct
title: RecursiveIteratorIterator::__construct
summary: Wraps a recursive cursor iterator and flattens its entry tree according to $mode.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: RecursiveIterator<K, T>
    description: Iterator wrapped or consumed by this operation.
  - name: mode
    type: int
    description: Mode selected from the values documented below.
    default: RecursiveIteratorIterator::LEAVES_ONLY
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
owner: std.spl.RecursiveIteratorIterator
visibility: public
modifiers: []
---

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::__construct()` wraps a recursive cursor iterator and flattens its entry tree according to $mode.

## Behavior

Wraps a recursive cursor iterator and flattens its entry tree according to $mode.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new RecursiveIteratorIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
