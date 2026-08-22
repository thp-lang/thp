---
kind: class
id: std.spl.SplStack
title: SplStack
summary: Provides last-in, first-out access to a linked sequence.
name: SplStack
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

`SplStack` provides last-in, first-out access to a linked sequence.

## Construction

| Method                                               | Description                                    |
| ---------------------------------------------------- | ---------------------------------------------- |
| [`__construct()`](thp:std.spl.SplStack::__construct) | Creates an empty stack in LIFO iteration mode. |

## Behavior

`push()` adds to the top and `pop()` removes from the top. The inherited FIFO/LIFO mode cannot change the stack’s LIFO removal semantics.

## Errors

Each member page documents the failure conditions relevant to that operation. Concrete THP error classes remain unsettled.

## Example

```thp
$stack = new SplStack<string>();
$stack->push("first");
$stack->push("second");
$value = $stack->pop();
```

`$value` is `"second"`.

## See also

- [SPL data structures](thp:std.dataStructures)
- [PHP `SplStack`](https://www.php.net/manual/en/class.splstack.php)
- [`SplDoublyLinkedList`](thp:std.spl.SplDoublyLinkedList)
