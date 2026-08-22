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
    description: The TIn type parameter.
  - name: TOut
    description: The TOut type parameter.
interfaces:
  - id: std.spl.RecursiveIterator
    arguments:
      - K
      - TOut
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`RecursiveRegexIterator` applies a regular-expression filter throughout a recursive iterator.

## Construction

| Method                                                             | Description                                                                            |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RecursiveRegexIterator::__construct) | Wraps a recursive iterator and fixes the transformed value type for the selected mode. |

## Behavior

The regular expression and immutable mode are applied consistently to parent
and child iterators. Invalid patterns fail during construction.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$matches = new RecursiveRegexIterator<int, string, string>($tree, "/\.thp$/");
```

The recursive view keeps values that match the THP filename suffix.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RecursiveRegexIterator`](https://www.php.net/manual/en/class.recursiveregexiterator.php)
