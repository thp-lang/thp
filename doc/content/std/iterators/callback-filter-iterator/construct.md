---
kind: method
id: std.spl.CallbackFilterIterator::__construct
title: CallbackFilterIterator::__construct
summary: Wraps the iterator and stores the callback used to test each current value.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<K, V>
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
owner: std.spl.CallbackFilterIterator
visibility: public
modifiers: []
---

[`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator)`::__construct()` wraps the iterator and stores the callback used to test each current value.

## Behavior

Wraps the iterator and stores the callback used to test each current value.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new CallbackFilterIterator($iterator, $callback);
```

The call uses the signature and defaults documented above.

## See also

- [`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator)
