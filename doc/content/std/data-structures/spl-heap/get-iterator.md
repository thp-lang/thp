---
kind: method
id: std.spl.SplHeap::getIterator
title: SplHeap::getIterator
summary: Returns values in extraction order without mutating the heap.
name: getIterator
order: 9
typeParameters: []
parameters: []
returns:
  type: Iterator<int, T>
  description: Returns values in extraction order without mutating the heap.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.SplHeap
visibility: public
modifiers: []
---

[`SplHeap`](thp:std.spl.SplHeap)`::getIterator()` returns values in extraction order without mutating the heap.

## Behavior

Returns values in extraction order without mutating the heap.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`SplHeap`](thp:std.spl.SplHeap)
