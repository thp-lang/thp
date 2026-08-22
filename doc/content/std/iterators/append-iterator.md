---
kind: class
id: std.spl.AppendIterator
title: AppendIterator
summary: Traverses multiple iterators sequentially.
name: AppendIterator
module: iterators
typeParameters:
  - name: K
    description: The common key type of the appended iterators.
  - name: V
    description: The common value type of the appended iterators.
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - K
      - V
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`AppendIterator` traverses multiple iterators sequentially.

## Construction

| Method                                                     | Description                             |
| ---------------------------------------------------------- | --------------------------------------- |
| [`__construct()`](thp:std.spl.AppendIterator::__construct) | Creates an empty sequence of iterators. |

## Behavior

Values from each appended iterator are exhausted before traversal advances to the next iterator. Appending after traversal starts requires a final THP rule.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$all = new AppendIterator<int, int>();
$all->append($first);
$all->append($second);
foreach ($all as $value) {
    print($value);
}
```

Values from `$first` appear before values from `$second`.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `AppendIterator`](https://www.php.net/manual/en/class.appenditerator.php)
