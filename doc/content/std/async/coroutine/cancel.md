---
kind: method
id: std.async.Coroutine::cancel
title: Coroutine::cancel
summary: Idempotently requests cancellation without skipping cleanup.
name: cancel
order: 8
typeParameters: []
parameters: []
returns:
  type: void
  description: This method does not return a value.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
version: "0.1"
owner: std.async.Coroutine
visibility: public
modifiers: []
---

[`Coroutine`](thp:std.async.Coroutine)`::cancel()` requests cancellation idempotently.

## Behavior

If queued, the body never starts. If running or suspended, the coroutine receives
`CancellationException` at its next suspension checkpoint. `finally` and `using`
cleanup execute. If the exception is caught and the body returns, the coroutine
completes successfully. Cancelling an already terminal coroutine has no effect.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->cancel();
```

The call uses the signature and defaults documented above.

## See also

- [`Coroutine`](thp:std.async.Coroutine)
