---
kind: interface
id: std.async.SchedulerInterface
title: SchedulerInterface
summary: Defines coroutine creation, suspension, and scheduling.
name: SchedulerInterface
module: async
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This interface is proposed and is not implemented in this repository. The
  runtime integration contract may change.
version: "0.1"
---

`SchedulerInterface` defines the operations a scheduler must provide to create,
suspend, enqueue, and time coroutines.

## Contract

Implementations manage when enqueued coroutines run and how a suspended
coroutine returns control to the scheduler. They must also arrange non-blocking
timers for `delay()` and `timeout()`, and cooperate with runtime I/O adapters
that suspend the current coroutine.

The interface does not prescribe a queue implementation or execution order.
Implementations must avoid starvation and must never execute two coroutines in
parallel on the scheduler thread.

## Example

The following function creates a coroutine and submits it to a scheduler:

```thp
function schedule<T>(
    SchedulerInterface $scheduler,
    callable $function,
    mixed ...$arguments,
): Coroutine<T> {
    $coroutine = $scheduler->newCoroutine<T>($function, ...$arguments);
    $scheduler->enqueue($coroutine);

    return $coroutine;
}
```

## See also

- [`scheduler_register()`](thp:std.async.scheduler_register)
- [`scheduler_register_default()`](thp:std.async.scheduler_register_default)
- [`scheduler_is_registered()`](thp:std.async.scheduler_is_registered)
- [`async()`](thp:std.async.async)
- [`await()`](thp:std.async.await)
- [`suspend()`](thp:std.async.suspend)
- [`delay()`](thp:std.async.delay)
- [True Async scheduler hook RFC](https://true-async.github.io/en/rfc.html)
