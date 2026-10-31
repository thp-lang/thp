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
  - name: flags
    type: int
    description: CachingIterator flags; FULL_CACHE retains visited entries.
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
availability: implemented
notice: This experimental recursive caching member is implemented.
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
