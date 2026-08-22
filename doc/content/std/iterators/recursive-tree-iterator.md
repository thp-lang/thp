---
kind: class
id: std.spl.RecursiveTreeIterator
title: RecursiveTreeIterator
summary: Formats recursive traversal with tree prefixes.
name: RecursiveTreeIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from recursive traversal.
  - name: T
    description: The T type parameter.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - string
constants:
  - name: PREFIX_LEFT
    type: int
    description: Selects the left prefix segment.
  - name: PREFIX_MID_HAS_NEXT
    type: int
    description: Selects the middle segment when siblings remain.
  - name: PREFIX_MID_LAST
    type: int
    description: Selects the middle segment for the last sibling.
  - name: PREFIX_END_HAS_NEXT
    type: int
    description: Selects the ending segment when siblings remain.
  - name: PREFIX_END_LAST
    type: int
    description: Selects the ending segment for the last sibling.
  - name: PREFIX_RIGHT
    type: int
    description: Selects the right prefix segment.
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`RecursiveTreeIterator` formats recursive traversal with tree prefixes.

## Construction

| Method                                                            | Description                                                       |
| ----------------------------------------------------------------- | ----------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RecursiveTreeIterator::__construct) | Wraps the recursive iterator and produces formatted string lines. |

## Behavior

Each output string combines configurable branch prefixes, the entry value, and
a postfix. `$mode` selects traversal order, while the prefix methods change only
formatting. The iterator always yields `string`.

## Differences from PHP

PHP exposes bypass flags that can make the current value cease to be a formatted
string. THP omits those flags so `Iterator<int, string>` remains true for every
configuration.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$treeText = new RecursiveTreeIterator<int, Node>($tree);
foreach ($treeText as $line) {
    print($line);
}
```

Each line includes the prefix representing its depth and sibling position.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RecursiveTreeIterator`](https://www.php.net/manual/en/class.recursivetreeiterator.php)
