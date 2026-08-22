---
kind: method
id: std.spl.SplHeap::top
title: SplHeap::top
summary: Returns the next value without removing it.
name: top
order: 4
typeParameters: []
parameters: []
returns:
  type: T
  description: Returns the next value without removing it.
errors:
  - description:
      The operation fails when the container is empty. Comparison or delegated
      runtime failures propagate where applicable.
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

[`SplHeap`](thp:std.spl.SplHeap)`::top()` returns the next value without removing it.

## Behavior

Returns the next value without removing it.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->top();
```

The call uses the signature and defaults documented above.

## See also

- [`SplHeap`](thp:std.spl.SplHeap)
