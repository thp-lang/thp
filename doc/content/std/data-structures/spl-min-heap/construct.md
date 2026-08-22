---
kind: method
id: std.spl.SplMinHeap::__construct
title: SplMinHeap::__construct
summary: Creates an empty minimum heap.
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
owner: std.spl.SplMinHeap
visibility: public
modifiers: []
---

[`SplMinHeap`](thp:std.spl.SplMinHeap)`::__construct()` creates an empty minimum heap.

## Behavior

Creates an empty minimum heap.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new SplMinHeap();
```

The call uses the signature and defaults documented above.

## See also

- [`SplMinHeap`](thp:std.spl.SplMinHeap)
