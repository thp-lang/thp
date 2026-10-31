---
kind: class
id: std.spl.EmptyIterator
title: EmptyIterator
summary: Represents an iterator that never yields a value.
name: EmptyIterator
module: iterators
typeParameters:
  - name: K
    description: The key type the empty iterator would expose.
  - name: V
    description: The value type the empty iterator would expose.
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

`EmptyIterator` represents an iterator that never yields a value.

## Construction

| Method                                                    | Description                                    |
| --------------------------------------------------------- | ---------------------------------------------- |
| [`__construct()`](thp:std.spl.EmptyIterator::__construct) | Creates an iterator that is already exhausted. |

## Behavior

`rewind()` leaves the iterator exhausted, `valid()` always returns `false`, and
`advance()` has no effect.

## Errors

`key()` and `value()` fail when the cursor is exhausted.

## Example

```thp
$empty = new EmptyIterator<int, string>();
$empty->rewind();
$valid = $empty->valid();
```

`$valid` is `false`.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `EmptyIterator`](https://www.php.net/manual/en/class.emptyiterator.php)
