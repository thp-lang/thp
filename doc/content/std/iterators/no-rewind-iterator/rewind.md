---
kind: method
id: std.spl.NoRewindIterator::rewind
title: NoRewindIterator::rewind
summary: Leaves the shared inner cursor unchanged.
name: rewind
order: 1
typeParameters: []
parameters: []
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

Does nothing, including on an exhausted or partially traversed cursor. It never
calls the inner `rewind()` and therefore cannot propagate an inner rewind
failure. The initial call from `foreach` is also ignored. Repeated loops resume
at the current entry: breaking does not advance, while a fully consumed cursor
stays exhausted. Other inherited operations still delegate and propagate failures.

## See also

- [`NoRewindIterator`](thp:std.spl.NoRewindIterator)
