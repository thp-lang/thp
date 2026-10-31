---
kind: class
id: std.spl.CallbackFilterIterator
title: CallbackFilterIterator
summary: Keeps values accepted by a callback.
name: CallbackFilterIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: V
    description: The filtered value type.
parent:
  id: std.spl.FilterIterator
  arguments:
    - K
    - V
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: This experimental adapter executes in the standalone compiler and reference VM.
version: "0.6"
---

`CallbackFilterIterator` keeps values accepted by a callback.

## Construction

| Method                                                             | Description                                                                 |
| ------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.CallbackFilterIterator::__construct) | Wraps the iterator and stores the callback used to test each current value. |

## Behavior

The callback receives the current `V` value then its `K` key and returns
`bool`. Accepted entries keep their original keys and relative order. The
adapter tests lazily and caches an accepted entry until `advance()`; repeated
`valid()` calls do not invoke the callback again. An empty inner iterator
never calls the callback. Its `K` and `V` types come from the iterator or
explicit generic arguments; an untyped empty literal needs an expected
iterator shape.

## Errors

Construction and callback creation can fail to allocate. Callback exceptions,
inner cursor failures, and allocation failures propagate without a partial
result or another advance of the failing entry. External cursor mutation
through `getInnerIterator()` while filtering is active has no coordinated
cache invalidation.

## Example

```thp
$even = new CallbackFilterIterator<int, int>($numbers, function (int $value, int $key): bool {
    return $value % 2 === 0;
});
```

Only even values are produced.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `CallbackFilterIterator`](https://www.php.net/manual/en/class.callbackfilteriterator.php)
