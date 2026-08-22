---
kind: class
id: std.spl.RecursiveIteratorIterator
title: RecursiveIteratorIterator
summary: Flattens a recursive iterator into depth-aware traversal.
name: RecursiveIteratorIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from recursive iterators.
  - name: T
    description: The T type parameter.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - T
constants:
  - name: LEAVES_ONLY
    type: int
    description: Yields only values without children.
  - name: SELF_FIRST
    type: int
    description: Yields a parent before its children.
  - name: CHILD_FIRST
    type: int
    description: Yields children before their parent.
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`RecursiveIteratorIterator` flattens a recursive iterator into depth-aware traversal.

## Construction

| Method                                                                | Description                                                                       |
| --------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RecursiveIteratorIterator::__construct) | Wraps a recursive cursor iterator and flattens its entry tree according to $mode. |

## Behavior

Traversal mode selects leaf-only, parent-before-children, or
children-before-parent ordering. Child iterators come from each current
`RecursiveEntry`; hooks run at deterministic traversal boundaries.

## Differences from PHP

PHP's `CATCH_GET_CHILD` flag can suppress failures while obtaining child
iterators. THP omits that flag: failures from a `RecursiveEntry` child
iterator propagate to the caller.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$flat = new RecursiveIteratorIterator<int, Node>($tree, RecursiveIteratorIterator::SELF_FIRST);
$flat->setMaxDepth(3);
```

Parents are produced before descendants, up to depth three.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RecursiveIteratorIterator`](https://www.php.net/manual/en/class.recursiveiteratoriterator.php)
