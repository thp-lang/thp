---
kind: method
id: std.spl.RecursiveFilterIterator::__construct
title: RecursiveFilterIterator::__construct
summary:
  Wraps a recursive iterator. Subclasses decide which complete recursive entries
  are yielded.
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
owner: std.spl.RecursiveFilterIterator
visibility: public
modifiers: []
---

[`RecursiveFilterIterator`](thp:std.spl.RecursiveFilterIterator)`::__construct()` wraps a recursive iterator. Subclasses decide which complete recursive entries are yielded.

## Behavior

Wraps a recursive iterator. Subclasses decide which complete recursive entries are yielded.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new RecursiveFilterIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveFilterIterator`](thp:std.spl.RecursiveFilterIterator)
