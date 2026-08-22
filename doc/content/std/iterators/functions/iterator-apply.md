---
kind: function
id: std.spl.iterator_apply
title: iterator_apply
summary: Calls a callback for each remaining iterator value.
name: iterator_apply
order: 5
typeParameters:
  - name: K
    description: The iterator key type.
  - name: T
    description: The T type parameter.
parameters:
  - name: iterator
    type: Iterator<K, T>
    description: Cursor iterator to advance through its remaining values.
  - name: callback
    type: callable
    description: Callable receiving the value, then $args.
  - name: args
    type: vector<mixed>
    description: Additional arguments forwarded each time.
    default: "[]"
returns:
  type: int
  description: The number of callback invocations.
errors:
  - description: Failures from cursor operations or $callback propagate and stop traversal.
related: []
status: experimental
availability: proposed
notice:
  This PHP-inspired function contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
module: iterators
---

`iterator_apply()` calls a callback for each remaining iterator value.

## Behavior

The function passes the current value followed by `$args` to the callback,
advances the cursor, and stops when the callback returns `false` or `valid()`
returns `false`. It never rewinds the iterator.

## Example

```thp
$calls = iterator_apply($iterator, function (string $value): bool {
    print($value);
    return true;
});
```

## See also

- [SPL functions](thp:std.dataStructures)
- [PHP `iterator_apply()`](https://www.php.net/manual/en/function.iterator-apply.php)
