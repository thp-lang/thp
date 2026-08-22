---
kind: class
id: std.spl.IteratorIterator
title: IteratorIterator
summary: Adapts a cursor iterator to the outer-iterator contract.
name: IteratorIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: V
    description: The value type preserved from the wrapped iterator.
interfaces:
  - id: std.spl.OuterIterator
    arguments:
      - K
      - V
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`IteratorIterator` adapts a cursor iterator to the outer-iterator contract.

## Construction

| Method                                                       | Description                                                                                 |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.IteratorIterator::__construct) | Wraps the supplied cursor iterator without changing its key, value, or exhaustion behavior. |

## Behavior

The adapter forwards values from the wrapped iterator without changing their
order or exhaustion behavior. Call `getIterator()` on an aggregate before
constructing this adapter.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$adapter = new IteratorIterator<int, int>($values);
$inner = $adapter->getInnerIterator();
```

The wrapped iterator remains inspectable.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `IteratorIterator`](https://www.php.net/manual/en/class.iteratoriterator.php)
