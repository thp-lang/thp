---
kind: module
id: std.exceptions
title: Exceptions
summary: PHP-inspired exception categories proposed for THP.
module: exceptions
order: 30
status: experimental
availability: proposed
notice:
  This hierarchy is proposed for PHP migration analysis and is not implemented.
  THP APIs must not name these classes as concrete failures until the hierarchy is
  accepted.
---

| Class                                                              | Parent                     |
| ------------------------------------------------------------------ | -------------------------- |
| [`LogicException`](thp:std.spl.LogicException)                     | `Exception`                |
| [`BadFunctionCallException`](thp:std.spl.BadFunctionCallException) | `LogicException`           |
| [`BadMethodCallException`](thp:std.spl.BadMethodCallException)     | `BadFunctionCallException` |
| [`DomainException`](thp:std.spl.DomainException)                   | `LogicException`           |
| [`InvalidArgumentException`](thp:std.spl.InvalidArgumentException) | `LogicException`           |
| [`LengthException`](thp:std.spl.LengthException)                   | `LogicException`           |
| [`OutOfRangeException`](thp:std.spl.OutOfRangeException)           | `LogicException`           |
| [`RuntimeException`](thp:std.spl.RuntimeException)                 | `Exception`                |
| [`OutOfBoundsException`](thp:std.spl.OutOfBoundsException)         | `RuntimeException`         |
| [`OverflowException`](thp:std.spl.OverflowException)               | `RuntimeException`         |
| [`RangeException`](thp:std.spl.RangeException)                     | `RuntimeException`         |
| [`UnderflowException`](thp:std.spl.UnderflowException)             | `RuntimeException`         |
| [`UnexpectedValueException`](thp:std.spl.UnexpectedValueException) | `RuntimeException`         |

## See also

- [SPL reference](thp:std.dataStructures)
- THP predefined exceptions
- [PHP SPL exceptions](https://www.php.net/manual/en/spl.exceptions.php)
