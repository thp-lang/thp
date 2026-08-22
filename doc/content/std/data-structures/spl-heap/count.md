---
kind: method
id: std.spl.SplHeap::count
title: SplHeap::count
summary: Returns the number of represented values.
name: count
order: 5
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the number of represented values.
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

[`SplHeap`](thp:std.spl.SplHeap)`::count()` returns the number of represented values.

## Behavior

Returns the number of represented values.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->count();
```

The call uses the signature and defaults documented above.

## See also

- [`SplHeap`](thp:std.spl.SplHeap)
