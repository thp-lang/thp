---
kind: interface
id: std.spl.SeekableIterator
title: SeekableIterator
summary: Positions an iterator cursor at a requested offset.
name: SeekableIterator
module: iterators
typeParameters:
  - name: K
    description: The iterator key type.
  - name: V
    description: The iterator value type.
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

`SeekableIterator<K, V>` lets a cursor iterator select its current position by
zero-based traversal offset.

## Contract

`seek()` positions the cursor at the requested zero-based traversal offset.
After a successful call, `valid()` is true and `key()` and `value()` expose the
selected element. Negative or unavailable positions fail; the concrete THP
error type is unsettled.

## Example

```thp
$iterator->seek(3);
$value = $iterator->value();
```

`$value` is the fourth traversal value.

## See also

- [SPL interfaces](thp:std.iterators)
- [PHP `SeekableIterator`](https://www.php.net/manual/en/class.seekableiterator.php)
