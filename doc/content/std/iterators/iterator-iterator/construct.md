---
kind: method
id: std.spl.IteratorIterator::__construct
title: IteratorIterator::__construct
summary:
  Wraps the supplied cursor iterator without changing its key, value, or exhaustion
  behavior.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<K, V>
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
owner: std.spl.IteratorIterator
visibility: public
modifiers: []
---

[`IteratorIterator`](thp:std.spl.IteratorIterator)`::__construct()` wraps the supplied cursor iterator without changing its key, value, or exhaustion behavior.

## Behavior

Wraps the supplied cursor iterator without changing its key, value, or exhaustion behavior.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new IteratorIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`IteratorIterator`](thp:std.spl.IteratorIterator)
