---
kind: method
id: std.async.Completable::isCancelled
title: Completable::isCancelled
summary: Returns true only when cancellation became the terminal state.
name: isCancelled
order: 3
typeParameters: []
parameters: []
returns:
  type: bool
  description: Returns true only when cancellation became the terminal state.
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
owner: std.async.Completable
visibility: public
modifiers: []
---

[`Completable`](thp:std.async.Completable)`::isCancelled()` returns true only when cancellation became the terminal state.

## Behavior

Returns true only when cancellation became the terminal state.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isCancelled();
```

The call uses the signature and defaults documented above.

## See also

- [`Completable`](thp:std.async.Completable)
