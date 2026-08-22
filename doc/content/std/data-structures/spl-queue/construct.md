---
kind: method
id: std.spl.SplQueue::__construct
title: SplQueue::__construct
summary: Creates an empty queue in FIFO iteration mode.
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
owner: std.spl.SplQueue
visibility: public
modifiers: []
---

[`SplQueue`](thp:std.spl.SplQueue)`::__construct()` creates an empty queue in FIFO iteration mode.

## Behavior

Creates an empty queue in FIFO iteration mode.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new SplQueue();
```

The call uses the signature and defaults documented above.

## See also

- [`SplQueue`](thp:std.spl.SplQueue)
