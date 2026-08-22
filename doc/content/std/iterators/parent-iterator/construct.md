---
kind: method
id: std.spl.ParentIterator::__construct
title: ParentIterator::__construct
summary: Wraps a recursive iterator and keeps entries whose children() value is not null.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: RecursiveIterator<K, T>
    description: Iterator wrapped or consumed by this operation.
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
owner: std.spl.ParentIterator
visibility: public
modifiers: []
---

[`ParentIterator`](thp:std.spl.ParentIterator)`::__construct()` wraps a recursive iterator and keeps entries whose children() value is not null.

## Behavior

Wraps a recursive iterator and keeps entries whose children() value is not null.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new ParentIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`ParentIterator`](thp:std.spl.ParentIterator)
