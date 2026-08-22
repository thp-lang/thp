---
kind: method
id: std.spl.SplHeap::extract
title: SplHeap::extract
summary: Removes and returns the highest-ranked value.
name: extract
order: 2
typeParameters: []
parameters: []
returns:
  type: T
  description: Removes and returns the highest-ranked value.
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

[`SplHeap`](thp:std.spl.SplHeap)`::extract()` removes and returns the highest-ranked value.

## Behavior

Removes and returns the highest-ranked value.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$result = $instance->extract();
```

The call uses the signature and defaults documented above.

## See also

- [`SplHeap`](thp:std.spl.SplHeap)
