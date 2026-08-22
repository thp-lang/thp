---
kind: method
id: std.spl.RecursiveTreeIterator::__construct
title: RecursiveTreeIterator::__construct
summary: Wraps the recursive iterator and produces formatted string lines.
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
    default: RecursiveIteratorIterator::SELF_FIRST
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
owner: std.spl.RecursiveTreeIterator
visibility: public
modifiers: []
---

[`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)`::__construct()` wraps the recursive iterator and produces formatted string lines.

## Behavior

Wraps the recursive iterator and produces formatted string lines.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new RecursiveTreeIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)
