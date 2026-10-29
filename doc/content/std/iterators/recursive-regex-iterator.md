---
kind: class
id: std.spl.RecursiveRegexIterator
title: RecursiveRegexIterator
summary: Applies a regular-expression filter throughout a recursive iterator.
name: RecursiveRegexIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: TIn
    description: Input node value type.
  - name: TOut
    description: Output node value type fixed by mode and flags.
interfaces:
  - id: std.spl.RecursiveIterator
    arguments:
      - K
      - TOut
constants:
  - name: MATCH
    type: int
    description: Filters matching values.
  - name: GET_MATCH
    type: int
    description: Yields first capture maps.
  - name: ALL_MATCHES
    type: int
    description: Yields all capture maps.
  - name: SPLIT
    type: int
    description: Yields split parts.
  - name: REPLACE
    type: int
    description: Yields replacement results.
  - name: USE_KEY
    type: int
    description: Filters by string keys.
properties: []
status: experimental
availability: implemented
notice: The VM applies the same constant modes and flags as RegexIterator to recursive entries.
version: "0.11"
---

`RecursiveRegexIterator` applies a regular-expression filter throughout a recursive iterator.

## Construction

| Method                                                             | Description                                                                            |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RecursiveRegexIterator::__construct) | Wraps a recursive iterator and fixes the transformed value type for the selected mode. |

## Behavior

The regular expression and immutable mode are applied to each node value and
all child iterators. A parent with children remains available for traversal
even when its own value does not match; leaf filters still apply. Result nodes
carry `RecursiveEntry<K, TOut>` values and wrapped child iterators. Modes,
flags, and output types follow [`RegexIterator`](thp:std.spl.RegexIterator).
Invalid patterns fail during construction.

## Errors

Cursor operations propagate failures from the wrapped iterator or PCRE2
engine at the cursor call site.

## Example

```thp
$matches = new RecursiveRegexIterator($tree, '/[.]thp$/');
```

The recursive view keeps values that match the THP filename suffix.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RecursiveRegexIterator`](https://www.php.net/manual/en/class.recursiveregexiterator.php)
