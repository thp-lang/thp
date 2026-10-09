---
kind: module
id: std.dataStructures
title: Data structures
summary: Generic containers, observer contracts, and object utilities.
module: data-structures
order: 60
status: experimental
availability: partial
notice:
  The THP-native classes listed below are implemented. The PHP-inspired SPL
  classes remain migration-analysis contracts or proposals.
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
| [`FixedSequence`](thp:std.dataStructures.FixedSequence)  | Native indexed nullable sequence.                |
| [`LinkedList`](thp:std.dataStructures.LinkedList)        | Native double-ended sequence.                    |
| [`Queue`](thp:std.dataStructures.Queue)                  | Native FIFO sequence.                            |
| [`Stack`](thp:std.dataStructures.Stack)                  | Native LIFO sequence.                            |
| [`MaxHeap`](thp:std.dataStructures.MaxHeap)              | Greatest-first value queue.                      |
| [`MinHeap`](thp:std.dataStructures.MinHeap)              | Least-first value queue.                         |
| [`PriorityQueue`](thp:std.dataStructures.PriorityQueue)  | Integer-priority value queue.                    |
| [`TypedMap`](thp:std.dataStructures.TypedMap)            | Mutable typed map wrapper.                       |
| [`ObjectStorage`](thp:std.dataStructures.ObjectStorage)  | Object-identity data association.                |
| [`ArrayObject`](thp:std.spl.ArrayObject)                 | Object wrapper around array-like storage.        |
| [`SplObjectStorage`](thp:std.spl.SplObjectStorage)       | Object-identity set with optional attached data. |

`SplFixedArray` and `ArrayObject` are retained only as PHP migration-analysis
placeholders. THP uses `FixedSequence<T>` for a sequence with an explicit size
and `TypedMap<K, V>` for a mutable map wrapper.

## See also

- [SPL reference](thp:std.dataStructures)
- [PHP SPL data structures](https://www.php.net/manual/en/spl.datastructures.php)

## Observer contracts

[`SplObserver`](thp:std.spl.SplObserver) receives notifications from
[`SplSubject`](thp:std.spl.SplSubject).
