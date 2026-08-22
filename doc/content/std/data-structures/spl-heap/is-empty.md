---
kind: method
id: std.spl.SplHeap::isEmpty
title: SplHeap::isEmpty
summary: Reports whether the container has no values.
name: isEmpty
order: 6
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the container has no values.
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

[`SplHeap`](thp:std.spl.SplHeap)`::isEmpty()` reports whether the container has no values.

## Behavior

Reports whether the container has no values.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isEmpty();
```

The call uses the signature and defaults documented above.

## See also

- [`SplHeap`](thp:std.spl.SplHeap)
