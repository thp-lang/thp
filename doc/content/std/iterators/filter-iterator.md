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
availability: implemented
notice: This experimental abstract adapter executes in the standalone compiler and reference VM.
version: "0.6"
---

This is an abstract class.

`FilterIterator` defines an iterator adapter that conditionally yields inner values.

## Construction

| Method                                                     | Description                                                     |
| ---------------------------------------------------------- | --------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.FilterIterator::__construct) | Wraps the iterator whose current values are tested by accept(). |

## Behavior

For each inner entry, `accept($value, $key)` decides whether it is yielded.
The adapter preserves the `K` key and `V` value of each accepted entry. It
rewinds its inner iterator when rewound, then tests entries lazily during
`valid()`, `key()`, or `value()`. A successful test is cached until `advance()`;
repeated `valid()` calls do not repeat that test. A rejected entry advances the
inner iterator. The adapter itself does not allocate a result collection.
An empty inner iterator calls no `accept()` method; its `K` and `V` types come
from the typed iterator or explicit constructor arguments. An untyped empty
literal without an expected iterator type is rejected.

## Errors

Construction and callback creation can fail to allocate. Failures from
`accept()` or inner cursor operations propagate unchanged. A failure during
testing leaves the inner cursor at the entry that failed. External cursor
mutation through `getInnerIterator()` while filtering is active has no
coordinated cache invalidation.

## Example

```thp
class EvenFilter extends FilterIterator<int, int> {
    public function accept(int $value, int $key): bool {
        return $value % 2 == 0;
    }
}
```

Concrete subclasses provide the acceptance rule.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `FilterIterator`](https://www.php.net/manual/en/class.filteriterator.php)
