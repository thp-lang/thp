---
kind: method
id: std.spl.NoRewindIterator::__construct
title: NoRewindIterator::__construct
summary: Retains the inner cursor at its current position.
name: __construct
order: 0
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<K, V>
    description: Cursor retained without repositioning.
returns:
  type: void
  description: This method does not return a value.
errors: []
related: []
status: experimental
availability: proposed
notice: This experimental method is proposed and unavailable at runtime.
version: "0.1"
owner: std.spl.NoRewindIterator
visibility: public
modifiers: []
---

Retains the supplied iterator object with its existing key and value types and
cursor state. Does not call `rewind()`, `valid()`, `key()`, `value()`, or
`advance()` and does not copy the cursor. Even an exhausted iterator can be
wrapped. Pass a cursor obtained from an aggregate rather than the aggregate.
Argument typing uses the existing typed collection-to-iterator conversion rules.
The adapter and every alias share cursor movement and exhaustion.

## See also

- [`NoRewindIterator`](thp:std.spl.NoRewindIterator)
