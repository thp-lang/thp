---
kind: class
id: std.spl.RegexIterator
title: RegexIterator
summary: Filters or transforms iterator values using a regular expression.
name: RegexIterator
module: iterators
typeParameters:
  - name: K
    description: The key type preserved from the wrapped iterator.
  - name: TIn
    description: The TIn type parameter.
  - name: TOut
    description: The TOut type parameter.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - TOut
constants:
  - name: MATCH
    type: int
    description: Filters by a regular-expression match.
  - name: GET_MATCH
    type: int
    description: Yields the first match details.
  - name: ALL_MATCHES
    type: int
    description: Yields all match details.
  - name: SPLIT
    type: int
    description: Yields regular-expression splits.
  - name: REPLACE
    type: int
    description: Yields replacement results.
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`RegexIterator<K, TIn, TOut>` filters or transforms iterator values using a
regular expression.

## Construction

| Method                                                    | Description                                                               |
| --------------------------------------------------------- | ------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RegexIterator::__construct) | Wraps the input iterator and fixes the output type for the selected mode. |

## Behavior

Mode controls whether matching preserves `TIn` values or produces a declared
capture, split, or replacement `TOut`. The mode is immutable because changing
it could invalidate the iterator's output type. The `$invert` constructor
argument selects non-matching inputs without overloading the mode. The output
preserves the input iterator's `K` keys. `USE_KEY` applies the pattern to each
current key while preserving both key and output types.

## Differences from PHP

PHP selects inverted matching with `INVERT_MATCH` and key matching with an
integer mode. THP exposes `$invert` as a boolean and key matching as the
explicit `RegexIterator::USE_KEY` mode while preserving the generic result
type.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$thpFiles = new RegexIterator<int, string, string>($paths, "/\.thp$/");
```

Only matching path strings are yielded in the default mode.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RegexIterator`](https://www.php.net/manual/en/class.regexiterator.php)
