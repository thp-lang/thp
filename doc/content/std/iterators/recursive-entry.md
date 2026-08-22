---
kind: class
id: std.spl.RecursiveEntry
title: RecursiveEntry
summary: Carries a value and the optional iterator for its children.
name: RecursiveEntry
module: iterators
typeParameters:
  - name: T
    description: The type of each recursive value.
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice: This is the intended THP-native replacement for PHP's implicit recursive
  iterator cursor. It is not implemented in this checkout.
version: "0.1"
---

This is a final class.

`RecursiveEntry<T>` keeps a recursive value and its children in one immutable
pull result.

## Behavior

The value and child source remain paired after the parent iterator advances.
Consumers never query child state through an implicit current cursor.

## Example

```thp
function isLeaf<T>(RecursiveEntry<T> $entry): bool {
    return $entry->children() === null;
}
```

## See also

- [`RecursiveIterator`](thp:std.spl.RecursiveIterator)
- [THP `Iterator`](thp:std.baseTypes.Iterator)
