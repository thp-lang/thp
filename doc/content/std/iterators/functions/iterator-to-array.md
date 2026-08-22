---
kind: function
id: std.spl.iterator_to_array
title: iterator_to_array
summary: Collects remaining iterator values into a vector.
name: iterator_to_array
order: 7
typeParameters:
  - name: K
    description: The iterator key type, ignored by this operation.
  - name: T
    description: The T type parameter.
parameters:
  - name: iterator
    type: Iterator<K, T>
    description: Cursor iterator to advance through its remaining values.
returns:
  type: vector<T>
  description:
    A vector containing the remaining values in traversal order. Keys are not
    retained.
errors:
  - description: Failures from cursor inspection or advancement propagate.
related: []
status: experimental
availability: proposed
notice:
  This PHP migration-analysis placeholder is not an accepted THP-native API and
  is not implemented. Native conversions will name their result shape.
version: "0.1"
module: iterators
---

`iterator_to_array()` records PHP migration behavior but is not a THP-native
name. The native vector-producing contract is planned as
`iterator_to_vector()`; a keyed conversion will use `iterator_to_map()`.

## Behavior

The function starts at the current cursor position and advances through
exhaustion without rewinding. Values are appended in traversal order and keys
are intentionally discarded.

## Example

```thp
$values = iterator_to_array($iterator);
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `iterator_to_array()`](https://www.php.net/manual/en/function.iterator-to-array.php)
