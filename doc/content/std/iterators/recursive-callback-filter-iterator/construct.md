---
kind: method
id: std.spl.RecursiveCallbackFilterIterator::__construct
title: RecursiveCallbackFilterIterator::__construct
summary: Wraps the recursive iterator and applies $callback to each RecursiveEntry.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: RecursiveIterator<K, T>
    description: Iterator wrapped or consumed by this operation.
  - name: callback
    type: callable
    description: Callable invoked by this operation.
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
owner: std.spl.RecursiveCallbackFilterIterator
visibility: public
modifiers: []
---

[`RecursiveCallbackFilterIterator`](thp:std.spl.RecursiveCallbackFilterIterator)`::__construct()` wraps the recursive iterator and applies $callback to each RecursiveEntry.

## Behavior

Wraps the recursive iterator and applies $callback to each RecursiveEntry.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new RecursiveCallbackFilterIterator($iterator, $callback);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveCallbackFilterIterator`](thp:std.spl.RecursiveCallbackFilterIterator)
