---
kind: class
id: std.spl.SplMaxHeap
title: SplMaxHeap
summary: Extracts the greatest inserted value first.
name: SplMaxHeap
module: data-structures
typeParameters:
  - name: T
    description: The T type parameter.
parent:
  id: std.spl.SplHeap
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

`SplMaxHeap` extracts the greatest inserted value first.

## Construction

| Method                                                 | Description                    |
| ------------------------------------------------------ | ------------------------------ |
| [`__construct()`](thp:std.spl.SplMaxHeap::__construct) | Creates an empty maximum heap. |

## Behavior

Values use their normal comparison order. `top()` observes and `extract()` removes the current maximum.

## Errors

Each member page documents the failure conditions relevant to that operation. Concrete THP error classes remain unsettled.

## Example

```thp
$heap = new SplMaxHeap<int>();
$heap->insert(3);
$heap->insert(8);
$maximum = $heap->extract();
```

`$maximum` is `8`.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `SplMaxHeap`](https://www.php.net/manual/en/class.splmaxheap.php)
- [`SplHeap`](thp:std.spl.SplHeap)
