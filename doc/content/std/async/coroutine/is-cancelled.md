---
kind: method
id: std.async.Coroutine::isCancelled
title: Coroutine::isCancelled
summary:
  Returns true when cancellation became the terminal state. State methods observe
  the coroutine at the instant they are called. Another scheduling turn may change a
  non-terminal state before the caller uses the result.
name: isCancelled
order: 7
typeParameters: []
parameters: []
returns:
  type: bool
  description:
    Returns true when cancellation became the terminal state. State methods
    observe the coroutine at the instant they are called. Another scheduling turn may
    change a non-terminal state before the caller uses the result.
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
owner: std.async.Coroutine
visibility: public
modifiers: []
---

[`Coroutine`](thp:std.async.Coroutine)`::isCancelled()` returns true when cancellation became the terminal state. State methods observe the coroutine at the instant they are called. Another scheduling turn may change a non-terminal state before the caller uses the result.

## Behavior

Returns true when cancellation became the terminal state. State methods observe the coroutine at the instant they are called. Another scheduling turn may change a non-terminal state before the caller uses the result.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isCancelled();
```

The call uses the signature and defaults documented above.

## See also

- [`Coroutine`](thp:std.async.Coroutine)
