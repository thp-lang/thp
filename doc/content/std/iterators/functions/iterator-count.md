---
kind: function
id: std.spl.iterator_count
title: iterator_count
summary: Counts and consumes the values remaining at an iterator's current cursor.
name: iterator_count
order: 6
typeParameters:
  - name: K
    description: The iterator key type.
  - name: V
    description: The iterator value type.
parameters:
  - name: iterator
    type: Iterator<K, V>
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

`iterator_count<K, V>(Iterator<K, V>): int` is a proposed consuming operation
on an explicit iterator. It is not an alias for, or overload of, executable
[`count()`](thp:std.baseTypes).

## Behavior

The function begins at the cursor's current position, counts through
exhaustion, and advances after every counted value. It does not call
`rewind()` before or after counting. A partially consumed iterator therefore
produces only its remaining count and is exhausted when the function returns.

By contrast, `count(string|vector<T>|map<K, V>): int` reads an existing value's
byte or collection length and has no traversal cursor to consume or move.

## Example

```thp
$remaining = iterator_count($iterator);
var_dump($iterator->valid()); // false
```

## See also

- [SPL functions](thp:std.dataStructures)
- [Base types and executable `count()`](thp:std.baseTypes)
- [PHP `iterator_count()`](https://www.php.net/manual/en/function.iterator-count.php)
