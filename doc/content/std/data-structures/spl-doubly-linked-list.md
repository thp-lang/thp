---
kind: class
id: std.spl.SplDoublyLinkedList
title: SplDoublyLinkedList
summary: Stores an indexed sequence with efficient operations at both ends.
name: SplDoublyLinkedList
module: data-structures
typeParameters:
  - name: T
    description: The T type parameter.
interfaces:
  - id: std.baseTypes.IteratorAggregate
    arguments:
      - int
      - T
  - id: std.baseTypes.Countable
  - id: std.baseTypes.MapAccess
    arguments:
      - int
      - T
constants:
  - name: IT_MODE_LIFO
    type: int
    description: Iterates from the end toward the beginning.
  - name: IT_MODE_FIFO
    type: int
    description: Iterates from the beginning toward the end.
  - name: IT_MODE_DELETE
    type: int
    description: Removes values as iteration advances.
  - name: IT_MODE_KEEP
    type: int
    description: Keeps values while iterating.
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`SplDoublyLinkedList` stores an indexed sequence with efficient operations at both ends.

## Construction

| Method                                                          | Description            |
| --------------------------------------------------------------- | ---------------------- |
| [`__construct()`](thp:std.spl.SplDoublyLinkedList::__construct) | Creates an empty list. |

## Behavior

Indices are contiguous and zero-based. End operations mutate the list. Iterator mode controls direction and whether traversal removes visited values; the default is FIFO with values kept.

## Differences from PHP

PHP exposes a mutable cursor and serialization methods on the list. THP uses
`getIterator()` for traversal and `toVector()` for an explicit value
snapshot; serialization is outside this contract.

## Errors

Reading or removing an unavailable index, or removing from an empty list, fails. Concrete THP error classes are not established.

## Example

```thp
$jobs = new SplDoublyLinkedList<string>();
$jobs->push("compile");
$jobs->unshift("lint");
$first = $jobs->shift();
```

`$first` is `"lint"` and one value remains.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `SplDoublyLinkedList`](https://www.php.net/manual/en/class.spldoublylinkedlist.php)
