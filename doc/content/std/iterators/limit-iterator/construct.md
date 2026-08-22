---
kind: method
id: std.spl.LimitIterator::__construct
title: LimitIterator::__construct
summary: Wraps the iterator, skips $offset values, and limits subsequent pulls.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<K, V>
    description: Iterator wrapped or consumed by this operation.
  - name: offset
    type: int
    description: Position addressed by the operation.
    default: "0"
  - name: limit
    type: int
    description: Maximum values to yield; -1 means unlimited.
    default: "-1"
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
owner: std.spl.LimitIterator
visibility: public
modifiers: []
---

[`LimitIterator`](thp:std.spl.LimitIterator)`::__construct()` wraps the iterator, skips $offset values, and limits subsequent pulls.

## Behavior

Wraps the iterator, skips $offset values, and limits subsequent pulls.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new LimitIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`LimitIterator`](thp:std.spl.LimitIterator)
