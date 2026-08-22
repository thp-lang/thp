---
kind: method
id: std.spl.RecursiveCachingIterator::__construct
title: RecursiveCachingIterator::__construct
summary: Wraps a recursive cursor iterator and optionally retains every visited recursive entry.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: RecursiveIterator<K, T>
    description: Iterator wrapped or consumed by this operation.
  - name: full_cache
    type: bool
    description: Retain every visited entry when true.
    default: "false"
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
owner: std.spl.RecursiveCachingIterator
visibility: public
modifiers: []
---

[`RecursiveCachingIterator`](thp:std.spl.RecursiveCachingIterator)`::__construct()` wraps a recursive cursor iterator and optionally retains every visited recursive entry.

## Behavior

Wraps a recursive cursor iterator and optionally retains every visited recursive entry.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new RecursiveCachingIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveCachingIterator`](thp:std.spl.RecursiveCachingIterator)
