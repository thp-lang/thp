---
kind: class
id: std.spl.SplQueue
title: SplQueue
summary: Provides first-in, first-out access to a linked sequence.
name: SplQueue
module: data-structures
typeParameters:
  - name: T
    description: The T type parameter.
parent:
  id: std.spl.SplDoublyLinkedList
  arguments:
    - T
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`SplQueue` provides first-in, first-out access to a linked sequence.

## Construction

| Method                                               | Description                                    |
| ---------------------------------------------------- | ---------------------------------------------- |
| [`__construct()`](thp:std.spl.SplQueue::__construct) | Creates an empty queue in FIFO iteration mode. |

## Behavior

`enqueue()` adds at the back and `dequeue()` removes from the front. Queue iteration remains FIFO.

## Errors

Each member page documents the failure conditions relevant to that operation. Concrete THP error classes remain unsettled.

## Example

```thp
$queue = new SplQueue<string>();
$queue->enqueue("first");
$queue->enqueue("second");
$value = $queue->dequeue();
```

`$value` is `"first"`.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `SplQueue`](https://www.php.net/manual/en/class.splqueue.php)
- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
