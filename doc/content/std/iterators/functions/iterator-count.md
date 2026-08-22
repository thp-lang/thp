---
kind: function
id: std.spl.iterator_count
title: iterator_count
summary: Counts the remaining values in an iterable.
name: iterator_count
order: 6
typeParameters:
  - name: K
    description: The iterator key type.
  - name: T
    description: The T type parameter.
parameters:
  - name: iterator
    type: Iterator<K, T>
    description: Cursor iterator to advance and count.
returns:
  type: int
  description: The number of values observed.
errors:
  - description: Failures from cursor inspection or advancement propagate.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired function contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
module: iterators
---

`iterator_count()` counts the remaining values in an iterable.

## Behavior

The iterator advances from its current state through exhaustion. The function
does not rewind it before or after counting.

## Example

```thp
$remaining = iterator_count($iterator);
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `iterator_count()`](https://www.php.net/manual/en/function.iterator-count.php)
