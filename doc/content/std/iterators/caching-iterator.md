---
kind: class
id: std.spl.CachingIterator
title: CachingIterator
summary: Adds lookahead and optional caching to another iterator.
name: CachingIterator
module: iterators
typeParameters:
  - name: K
    description: The K type parameter.
  - name: V
    description: The V type parameter.
interfaces:
  - id: std.spl.OuterIterator
    arguments:
      - K
      - V
  - id: std.baseTypes.MapAccess
    arguments:
      - K
      - V
  - id: std.baseTypes.Countable
  - id: std.baseTypes.Stringable
constants:
  - name: CALL_TOSTRING
    type: int
    description: Converts each visited value to a string.
  - name: CATCH_GET_CHILD
    type: int
    description: Suppresses failures while checking recursive children.
  - name: TOSTRING_USE_KEY
    type: int
    description: Uses each entry key for string conversion.
  - name: TOSTRING_USE_CURRENT
    type: int
    description: Uses each entry value for string conversion.
  - name: TOSTRING_USE_INNER
    type: int
    description: Uses the inner iterator’s string value.
  - name: FULL_CACHE
    type: int
    description: Retains every visited key and value.
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`CachingIterator` adds lookahead and optional caching to another iterator.

## Construction

| Method                                                      | Description                                                                                      |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| [`__construct()`](thp:std.spl.CachingIterator::__construct) | Wraps the keyed iterator and configures lookahead, string conversion, and optional full caching. |

## Behavior

`hasNext()` provides one-value lookahead. Cache access is available only when full caching is enabled. String conversion behavior depends on flags.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$cached = new CachingIterator<int, string>($iterator, CachingIterator::FULL_CACHE);
foreach ($cached as $value) {
    print($value->value());
}
$copy = $cached->getCache();
```

After traversal, `$copy` contains the cached values.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `CachingIterator`](https://www.php.net/manual/en/class.cachingiterator.php)
