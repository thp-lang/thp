---
kind: method
id: std.spl.CachingIterator::__construct
title: CachingIterator::__construct
summary:
  Wraps the keyed iterator and configures lookahead, string conversion, and
  optional full caching.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<K, V>
    description: Iterator wrapped or consumed by this operation.
  - name: flags
    type: int
    description: Bit mask selecting the documented options.
    default: CachingIterator::CALL_TOSTRING
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
owner: std.spl.CachingIterator
visibility: public
modifiers: []
---

[`CachingIterator`](thp:std.spl.CachingIterator)`::__construct()` wraps the keyed iterator and configures lookahead, string conversion, and optional full caching.

## Behavior

Wraps the keyed iterator and configures lookahead, string conversion, and optional full caching.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new CachingIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`CachingIterator`](thp:std.spl.CachingIterator)
