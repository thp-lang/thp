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
    type: callable<V, K, bool>
    description: Receives each current value then key and decides acceptance.
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
notice: This experimental constructor executes in the standalone compiler and reference VM.
version: "0.6"
owner: std.spl.CallbackFilterIterator
visibility: public
modifiers: []
---

[`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator)`::__construct()` wraps the iterator and stores the callback used to test each current value.

## Behavior

Wraps the iterator and stores the typed callback used to test each current
value and key. The adapter keeps the inner cursor; it does not rewind it until
`rewind()` is called. Allocation failure propagates.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new CallbackFilterIterator($iterator, $callback);
```

The call uses the signature and defaults documented above.

## See also

- [`CallbackFilterIterator`](thp:std.spl.CallbackFilterIterator)
