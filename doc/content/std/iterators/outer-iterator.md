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
availability: proposed
notice:
  This PHP-inspired interface contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`OuterIterator` exposes the iterator wrapped by an iterator adapter.

## Contract

Implementations wrap another iterator and return that same logical iterator from `getInnerIterator()`. Replacing the inner iterator after construction is not part of this contract.

## Example

```thp
function inner<K, V>(OuterIterator<K, V> $iterator): ?Iterator<K, V> {
    return $iterator->getInnerIterator();
}
```

The caller can inspect an adapter without depending on its concrete class.

## See also

- [SPL interfaces](thp:std.iterators)
- [PHP `OuterIterator`](https://www.php.net/manual/en/class.outeriterator.php)
