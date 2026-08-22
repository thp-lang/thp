---
kind: class
id: std.spl.ArrayIterator
title: ArrayIterator
summary: Iterates and mutates map-like storage.
name: ArrayIterator
module: iterators
typeParameters:
  - name: K
    description: The K type parameter.
  - name: V
    description: The V type parameter.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - V
  - id: std.spl.SeekableIterator
    arguments:
      - K
      - V
  - id: std.baseTypes.MapAccess
    arguments:
      - K
      - V
  - id: std.baseTypes.Countable
constants:
  - name: STD_PROP_LIST
    type: int
    description: Uses object properties for property-list operations.
  - name: ARRAY_AS_PROPS
    type: int
    description: Allows stored keys to be accessed as properties.
properties: []
status: experimental
availability: proposed
notice:
  This PHP migration-analysis placeholder is not an accepted THP-native API and
  is not implemented. The native map-backed iterator is planned as MapIterator.
version: "0.1"
---

`ArrayIterator` records the PHP migration shape of an iterator over map-like
storage. THP does not have an `array` type; the native contract is planned as
`MapIterator<K, V>`.

## Construction

| Method                                                    | Description                                                                      |
| --------------------------------------------------------- | -------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.ArrayIterator::__construct) | Copies the supplied map or public object properties into keyed iterator storage. |

## Behavior

Iteration follows the wrapped storage order. Offset mutations and sorting affect subsequent traversal. THP uses typed map storage rather than PHP’s untyped array.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$iterator = new ArrayIterator<string, int>({"low" => 1, "high" => 9});
$iterator->seek(1);
echo $iterator->key() . "=" . $iterator->value();
```

The cursor position selected by `seek()` has key `"high"` and value `9`.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `ArrayIterator`](https://www.php.net/manual/en/class.arrayiterator.php)
