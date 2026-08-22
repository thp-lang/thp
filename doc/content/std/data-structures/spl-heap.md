---
kind: class
id: std.spl.SplHeap
title: SplHeap
summary: Defines a value-ordered binary heap.
name: SplHeap
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
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

This is an abstract class.

`SplHeap` defines a value-ordered binary heap.

## Construction

`SplHeap` is abstract and cannot be constructed directly. Concrete subclasses
create an empty heap with their comparison policy.

## Behavior

The greatest value according to `compare()` is exposed first. Extraction mutates the heap. Equal comparisons do not establish stable insertion order.

## Errors

Each member page documents the failure conditions relevant to that operation. Concrete THP error classes remain unsettled.

## Example

```thp
function inspect<T>(SplHeap<T> $heap): ?T {
    return $heap->isEmpty() ? null : $heap->top();
}
```

Abstract heaps are consumed through a concrete comparison policy.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `SplHeap`](https://www.php.net/manual/en/class.splheap.php)
