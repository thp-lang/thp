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
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`CallbackFilterIterator` keeps values accepted by a callback.

## Construction

| Method                                                             | Description                                                                 |
| ------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.CallbackFilterIterator::__construct) | Wraps the iterator and stores the callback used to test each current value. |

## Behavior

The callback receives each current value and decides whether it is retained.
Callback failures propagate and stop traversal.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$even = new CallbackFilterIterator<int, int>($numbers, function (int $value): bool {
    return $value % 2 === 0;
});
```

Only even values are produced.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `CallbackFilterIterator`](https://www.php.net/manual/en/class.callbackfilteriterator.php)
