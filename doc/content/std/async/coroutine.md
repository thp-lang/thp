---
kind: class
id: std.async.Coroutine
title: Coroutine
summary: Represents a scheduled callable and its eventual typed result.
name: Coroutine
module: async
typeParameters:
  - name: T
    description: The callable's successful return value type.
interfaces:
  - id: std.async.Completable
    arguments:
      - T
constants: []
properties: []
status: experimental
availability: proposed
notice:
  Coroutine is a proposed runtime class and is not implemented in this repository.
  State transitions and cancellation failure types may change.
version: "0.1"
---

This is a final class.

`Coroutine<T>` is the handle returned by [`async()`](thp:std.async.async). It exposes the
lifecycle of one scheduled callable and implements the one-shot
[`Completable<T>`](thp:std.async.Completable) contract.

## Construction

`Coroutine` has no public constructor. [`async()`](thp:std.async.async) asks the active
scheduler to create and enqueue it.

## Lifecycle

A coroutine moves through these observable states:

1. **Queued** — accepted by the scheduler but not executing.
2. **Running** — currently executing on the scheduler thread.
3. **Suspended** — waiting at `await()`, `delay()`, `suspend()`, or supported
   non-blocking I/O.
4. **Completed** — finished with a value or failure.
5. **Cancelled** — cancellation became its terminal outcome.

Queued, running, and suspended are non-terminal scheduling states. Completed
and cancelled are terminal. The scheduler may move a coroutine between running
and suspended many times.

## Errors

The callable's throwable is stored as the coroutine's failed outcome and is
re-thrown by [`await()`](thp:std.async.await). Cancellation produces a distinct failure,
but its concrete type is not yet established.

Dropping the last handle to a failed, unobserved coroutine must report the
failure through a runtime-level unhandled-coroutine mechanism. The exact
reporting hook remains part of the runtime design.

## Example

```thp
scheduler_register_default();

$request: Coroutine<string> = async(function (): string {
    delay(10);
    return fetchText("/status");
});

if (!$request->isCompleted()) {
    echo "request pending";
}

echo await($request);
```

## See also

- [`Completable<T>`](thp:std.async.Completable)
- [`async()`](thp:std.async.async)
- [`await()`](thp:std.async.await)
- [`delay()`](thp:std.async.delay)
- [True Async Coroutine](https://true-async.github.io/en/docs/components/coroutines.html)
