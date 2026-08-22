---
kind: class
id: std.spl.ParentIterator
title: ParentIterator
summary: Keeps only recursive values that have children.
name: ParentIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: T
    description: The T type parameter.
parent:
  id: std.spl.RecursiveFilterIterator
  arguments:
    - K
    - T
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`ParentIterator` keeps only recursive values that have children.

## Construction

| Method                                                     | Description                                                                      |
| ---------------------------------------------------------- | -------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.ParentIterator::__construct) | Wraps a recursive iterator and keeps entries whose children() value is not null. |

## Behavior

`accept()` tests `$entry->children() !== null`. Leaf entries are skipped.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$parents = new ParentIterator<int, Node>($tree);
foreach ($parents as $node) {
    print($node->value()->name);
}
```

Only nodes with children are produced.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `ParentIterator`](https://www.php.net/manual/en/class.parentiterator.php)
