---
kind: method
id: std.baseTypes.Iterator::advance
title: Iterator::advance
summary: Moves the cursor to the next element.
name: advance
order: 5
typeParameters: []
parameters: []
returns:
  type: void
  description: This method does not return a value.
errors:
  - description: Failures encountered while advancing the underlying source propagate.
related: []
status: experimental
availability: proposed
notice: This member belongs to an experimental API contract and is not implemented in this repository.
version: "0.1"
owner: std.baseTypes.Iterator
visibility: public
modifiers: []
---

[`Iterator`](thp:std.baseTypes.Iterator)`::advance()` moves the cursor to the
next element.

## Behavior

When another element exists, it becomes the current element. Otherwise the
iterator becomes exhausted and `valid()` returns `false`. Calling `advance()`
after exhaustion leaves the iterator exhausted.

`foreach` calls this after the body and after `continue`. It does not call it
after `break`, `return`, or a throw. Any throwable propagates unchanged after
enclosing `using` and `finally` cleanup.

## Example

```thp
while ($iterator->valid()) {
    echo $iterator->value();
    $iterator->advance();
}
```

## See also

- [`Iterator`](thp:std.baseTypes.Iterator)
- [`valid()`](thp:std.baseTypes.Iterator::valid)
