---
kind: module
id: std.dataStructures
title: Data structures
summary: Generic containers, observer contracts, and object utilities.
module: data-structures
order: 60
status: experimental
availability: proposed
notice:
  These PHP-inspired class contracts are not implemented. Generic types, iteration
  behavior, serialization, and concrete errors may change.
---

| Class                                                    | Purpose                                          |
| -------------------------------------------------------- | ------------------------------------------------ |
| [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList) | Indexed double-ended linked sequence.            |
| [`SplStack`](thp:std.spl.SplStack)                       | Last-in, first-out stack.                        |
| [`SplQueue`](thp:std.spl.SplQueue)                       | First-in, first-out queue.                       |
| [`SplHeap`](thp:std.spl.SplHeap)                         | Base class for value-ordered heaps.              |
| [`SplMaxHeap`](thp:std.spl.SplMaxHeap)                   | Heap that extracts the greatest value first.     |
| [`SplMinHeap`](thp:std.spl.SplMinHeap)                   | Heap that extracts the smallest value first.     |
| [`SplPriorityQueue`](thp:std.spl.SplPriorityQueue)       | Heap ordered by explicit priorities.             |
| [`SplFixedArray`](thp:std.spl.SplFixedArray)             | Contiguous storage with an explicit size.        |
| [`ArrayObject`](thp:std.spl.ArrayObject)                 | Object wrapper around array-like storage.        |
| [`SplObjectStorage`](thp:std.spl.SplObjectStorage)       | Object-identity set with optional attached data. |

`SplFixedArray` and `ArrayObject` are retained only as PHP migration-analysis
placeholders. Their names are not accepted THP-native API names because THP has
separate `vector<T>` and `map<K, V>` types. A fixed-size sequence and typed map
wrapper require independently named contracts before implementation.

## See also

- [SPL reference](thp:std.dataStructures)
- [PHP SPL data structures](https://www.php.net/manual/en/spl.datastructures.php)

## Observer contracts

[`SplObserver`](thp:std.spl.SplObserver) receives notifications from
[`SplSubject`](thp:std.spl.SplSubject).
