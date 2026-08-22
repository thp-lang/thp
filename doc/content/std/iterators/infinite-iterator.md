---
kind: class
id: std.spl.InfiniteIterator
title: InfiniteIterator
summary: Repeats a finite inner iterator indefinitely.
name: InfiniteIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from each fresh iterator.
  - name: V
    description: The repeated value type.
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

`InfiniteIterator` repeatedly requests finite iterators from an aggregate.

## Construction

| Method                                                       | Description                                                                                                 |
| ------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.InfiniteIterator::__construct) | Stores the replayable aggregate. A fresh iterator is requested whenever the previous iterator is exhausted. |

## Behavior

After the current iterator is exhausted, traversal calls
`$source->getIterator()` and continues with the fresh iterator. If a fresh
iterator is empty, the infinite iterator becomes exhausted rather than polling
forever.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$repeat = new InfiniteIterator<int, string>($colors);
foreach (new LimitIterator<int, string>($repeat, limit: 5) as $color) {
    print($color);
}
```

The limit prevents the repeating iterator from running forever.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `InfiniteIterator`](https://www.php.net/manual/en/class.infiniteiterator.php)
