---
kind: class
id: std.spl.SplPriorityQueue
title: SplPriorityQueue
summary: Orders values by separately supplied priorities.
name: SplPriorityQueue
module: data-structures
typeParameters:
  - name: T
    description: The T type parameter.
  - name: P
    description: The P type parameter.
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

`SplPriorityQueue` orders values by separately supplied priorities.

## Construction

| Method                                                       | Description                      |
| ------------------------------------------------------------ | -------------------------------- |
| [`__construct()`](thp:std.spl.SplPriorityQueue::__construct) | Creates an empty priority queue. |

## Behavior

Higher priorities are extracted first. Equal priorities do not promise stable
insertion order. Unlike PHP's mode-dependent cursor, the THP-native contract
always yields stored values; priorities affect ordering but not result types.

## Differences from PHP

PHP extraction flags can return data, priority, or both and therefore change
the result shape. THP always returns the stored value; priorities only determine
ordering.

## Errors

Each member page documents the failure conditions relevant to that operation. Concrete THP error classes remain unsettled.

## Example

```thp
$queue = new SplPriorityQueue<string, int>();
$queue->insert("normal", 1);
$queue->insert("urgent", 10);
$next = $queue->extract();
```

`$next` is `"urgent"`; extraction always returns the stored value.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `SplPriorityQueue`](https://www.php.net/manual/en/class.splpriorityqueue.php)
