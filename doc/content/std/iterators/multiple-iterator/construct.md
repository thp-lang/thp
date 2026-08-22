---
kind: method
id: std.spl.MultipleIterator::__construct
title: MultipleIterator::__construct
summary: Creates an empty lockstep iterator. Attached iterators are returned in
  attachment order.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: require_all
    type: bool
    description: Whether exhaustion of any input ends traversal.
    default: "true"
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
owner: std.spl.MultipleIterator
visibility: public
modifiers: []
---

[`MultipleIterator`](thp:std.spl.MultipleIterator)`::__construct()` creates an empty lockstep iterator. Attached iterators are returned in attachment order.

## Behavior

Creates an empty lockstep iterator. Attached iterators are returned in attachment order.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new MultipleIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`MultipleIterator`](thp:std.spl.MultipleIterator)
