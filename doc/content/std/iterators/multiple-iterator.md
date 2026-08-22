---
kind: class
id: std.spl.MultipleIterator
title: MultipleIterator
summary: Advances multiple iterators in lockstep.
name: MultipleIterator
module: iterators
typeParameters:
  - name: T
    description: The T type parameter.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - int
      - vector<Option<T>>
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`MultipleIterator` advances multiple iterators in lockstep.

## Construction

| Method                                                       | Description                                                                              |
| ------------------------------------------------------------ | ---------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.MultipleIterator::__construct) | Creates an empty lockstep iterator. Attached iterators are returned in attachment order. |

## Behavior

Each cursor position combines the current values of all attached iterators.
Advancement moves every attached iterator once. When `$require_all` is `true`,
the first exhausted input ends traversal. Otherwise traversal continues to the
longest input and uses `Option::none()` for exhausted positions.

## Differences from PHP

PHP combines exhaustion and key policy in integer flags. THP uses the
`$require_all` boolean, preserves attachment order, and represents missing
positions with `Option::none()`.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$rows = new MultipleIterator<mixed>();
$rows->attachIterator($names);
$rows->attachIterator($emails);
```

Each produced row contains the corresponding name and email as `Option`
values, so the same row type works with either exhaustion policy.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `MultipleIterator`](https://www.php.net/manual/en/class.multipleiterator.php)
