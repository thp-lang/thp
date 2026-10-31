---
kind: interface
id: std.async.Completable
title: Completable
summary: Defines a cancellable asynchronous result that completes exactly once.
name: Completable
module: async
typeParameters:
  - name: T
    description: The value produced by successful completion.
interfaces:
  - id: std.async.Awaitable
    arguments:
      - T
constants: []
properties: []
status: experimental
availability: implemented
version: "0.1"
---

`Completable<T>` represents a one-shot asynchronous operation that eventually
succeeds with `T`, fails, or is cancelled.

## Contract

A completable transitions to one terminal state exactly once:

- successful, with a value of type `T`;
- failed, with the operation's throwable; or
- cancelled, with the Async runtime's cancellation failure.

Once terminal, its result does not change. Multiple calls to
[`await()`](thp:std.async.await) observe the same value or failure.

[`subscribe(callable<void> $notify): callable<void>`](thp:std.async.Completable::subscribe) registers a notification
for terminal completion. It invokes `$notify` exactly once, immediately when
already terminal. The returned callable unsubscribes idempotently. [`result(): T`](thp:std.async.Completable::result)
returns the stored value, rethrows the stored failure, or throws `Error` while
pending. Neither method can suspend. `await()` accepts `Completable<T>`;
[`Awaitable<T>`](thp:std.async.Awaitable) remains a marker.

Cancellation of a coroutine is cooperative. A queued coroutine is cancelled
before its body starts; a running or suspended coroutine receives
`CancellationException` at its next suspension checkpoint. If it catches the
exception and returns, it succeeds.

## Implementations

| Type                                      | Notes                                  |
| ----------------------------------------- | -------------------------------------- |
| [`Coroutine<T>`](thp:std.async.Coroutine) | Executes a callable under a scheduler. |
| [`Timeout`](thp:std.async.Timeout)        | Becomes ready after a duration.        |

## Example

```thp
function waitUnlessCancelled<T>(
    Completable<T> $operation,
    Completable<mixed> $cancellation,
): T {
    return await($operation, $cancellation);
}
```

Completion of `$cancellation` stops this wait. It does not automatically call
`$operation->cancel()`.

## See also

- [`Awaitable<T>`](thp:std.async.Awaitable)
- [`Coroutine<T>`](thp:std.async.Coroutine)
- [`await()`](thp:std.async.await)
- [`timeout()`](thp:std.async.timeout)
- [True Async base interfaces](https://true-async.github.io/en/docs/components/interfaces.html)
