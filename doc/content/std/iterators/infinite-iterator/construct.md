---
kind: method
id: std.spl.InfiniteIterator::__construct
title: InfiniteIterator::__construct
summary:
  Stores the replayable aggregate. A fresh iterator is requested whenever the
  previous iterator is exhausted.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: source
    type: IteratorAggregate<K, V>
    description: Value supplied as $source.
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
owner: std.spl.InfiniteIterator
visibility: public
modifiers: []
---

[`InfiniteIterator`](thp:std.spl.InfiniteIterator)`::__construct()` stores the replayable aggregate. A fresh iterator is requested whenever the previous iterator is exhausted.

## Behavior

Stores the replayable aggregate. A fresh iterator is requested whenever the previous iterator is exhausted.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new InfiniteIterator($source);
```

The call uses the signature and defaults documented above.

## See also

- [`InfiniteIterator`](thp:std.spl.InfiniteIterator)
