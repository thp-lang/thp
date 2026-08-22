---
kind: method
id: std.spl.SplMaxHeap::__construct
title: SplMaxHeap::__construct
summary: Creates an empty maximum heap.
name: __construct
order: 1
typeParameters: []
parameters: []
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Construction fails when an argument violates the documented contract or an
      underlying resource cannot be created. Concrete THP error classes remain
      experimental unless named above.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.SplMaxHeap
visibility: public
modifiers: []
---

[`SplMaxHeap`](thp:std.spl.SplMaxHeap)`::__construct()` creates an empty maximum heap.

## Behavior

Creates an empty maximum heap.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new SplMaxHeap();
```

The call uses the signature and defaults documented above.

## See also

- [`SplMaxHeap`](thp:std.spl.SplMaxHeap)
