---
kind: interface
id: std.spl.OuterIterator
title: OuterIterator
summary: Exposes the iterator wrapped by an iterator adapter.
name: OuterIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: V
    description: The value type preserved from the wrapped iterator.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - V
constants: []
properties: []
status: experimental
availability: implemented
notice: This experimental contract is implemented in the standalone compiler and VM.
version: "0.5"
---

`OuterIterator` exposes the iterator wrapped by an iterator adapter.

## Contract

Implementations wrap another iterator and return that same logical iterator from `getInnerIterator()`. Replacing the inner iterator after construction is not part of this contract.

[`getInnerIterator()`](thp:std.spl.OuterIterator::getInnerIterator)
returns a non-null `Iterator<K, V>`.

## Example

```thp
$outer = new IteratorIterator(new VectorIterator([1, 2]));
$inner: Iterator<int, int> = $outer->getInnerIterator();
```

The caller can inspect an adapter without depending on its concrete class.

## See also

- [SPL interfaces](thp:std.iterators)
- [PHP `OuterIterator`](https://www.php.net/manual/en/class.outeriterator.php)
