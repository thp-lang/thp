---
kind: class
id: std.spl.RecursiveCallbackFilterIterator
title: RecursiveCallbackFilterIterator
summary: Filters recursive values and their child iterators with a callback.
name: RecursiveCallbackFilterIterator
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

`RecursiveCallbackFilterIterator` filters recursive values and their child iterators with a callback.

## Construction

| Method                                                                      | Description                                                                |
| --------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RecursiveCallbackFilterIterator::__construct) | Wraps the recursive iterator and applies $callback to each RecursiveEntry. |

## Behavior

The callback decides whether an entry is yielded. It receives the complete
entry, so it may inspect both `value()` and whether `children()` is present.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$visible = new RecursiveCallbackFilterIterator<int, Node>($tree, function (RecursiveEntry<Node> $entry): bool {
    return !$entry->value()->hidden;
});
```

Hidden nodes are excluded from the recursive view.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RecursiveCallbackFilterIterator`](https://www.php.net/manual/en/class.recursivecallbackfilteriterator.php)
