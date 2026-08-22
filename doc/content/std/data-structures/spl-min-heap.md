---
kind: class
id: std.spl.SplMinHeap
title: SplMinHeap
summary: Extracts the smallest inserted value first.
name: SplMinHeap
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

`SplMinHeap` extracts the smallest inserted value first.

## Construction

| Method                                                 | Description                    |
| ------------------------------------------------------ | ------------------------------ |
| [`__construct()`](thp:std.spl.SplMinHeap::__construct) | Creates an empty minimum heap. |

## Behavior

Values use their normal comparison order. `top()` observes and `extract()` removes the current minimum.

## Errors

Each member page documents the failure conditions relevant to that operation. Concrete THP error classes remain unsettled.

## Example

```thp
$heap = new SplMinHeap<int>();
$heap->insert(3);
$heap->insert(8);
$minimum = $heap->extract();
```

`$minimum` is `3`.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `SplMinHeap`](https://www.php.net/manual/en/class.splminheap.php)
- [`SplHeap`](thp:std.spl.SplHeap)
