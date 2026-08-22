---
kind: class
id: std.spl.FilterIterator
title: FilterIterator
summary: Defines an iterator adapter that conditionally yields inner values.
name: FilterIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: V
    description: The filtered value type.
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

This is an abstract class.

`FilterIterator` defines an iterator adapter that conditionally yields inner values.

## Construction

| Method                                                     | Description                                                     |
| ---------------------------------------------------------- | --------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.FilterIterator::__construct) | Wraps the iterator whose current values are tested by accept(). |

## Behavior

For each inner value, `accept($value)` determines whether the adapter yields or
skips it. Implementations must keep filtering side effects predictable because
the method runs during advancement.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
function consume<K, V>(FilterIterator<K, V> $values): void {
    foreach ($values as $value) {
        print($value);
    }
}
```

Concrete subclasses provide the acceptance rule.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `FilterIterator`](https://www.php.net/manual/en/class.filteriterator.php)
