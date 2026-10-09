---
kind: class
id: std.spl.LimitIterator
title: LimitIterator
summary: Yields a bounded window of another iterator.
name: LimitIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: V
    description: The limited value type.
interfaces:
  - id: std.spl.OuterIterator
    arguments:
      - K
      - V
  - id: std.spl.SeekableIterator
    arguments:
      - K
      - V
constants: []
properties: []
status: experimental
availability: implemented
notice: This adapter executes through the common Iterator<K, V> cursor protocol.
version: "0.1"
---

`LimitIterator` yields a bounded window of another iterator.

## Construction

| Method                                                    | Description                                                            |
| --------------------------------------------------------- | ---------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.LimitIterator::__construct) | Wraps the iterator, skips $offset values, and limits subsequent pulls. |

## Behavior

Traversal skips the initial offset and yields at most `$limit` values; `-1` means no upper limit. Positions are relative to the limited view.

`seek()` rewinds and advances the wrapped iterator to the requested view
position. It fails if that position is not available.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$page = new LimitIterator<int, string>($records, offset: 20, limit: 10);
```

The iterator exposes at most ten records beginning at source position twenty.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `LimitIterator`](https://www.php.net/manual/en/class.limititerator.php)
