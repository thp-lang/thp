---
kind: module
id: std.async
title: Async
summary: Concurrency primitives and cooperatively scheduled operations.
module: async
order: 10
status: experimental
availability: proposed
notice:
  This section proposes an API that is not implemented in this repository. Names,
  signatures, cancellation, and failure behavior may change.
---

THP's proposed Async API runs ordinary callables in cooperatively scheduled
coroutines. A scheduler must be registered before starting asynchronous work.
Applications can select the built-in scheduler or provide an implementation of
[`SchedulerInterface`](thp:std.async.SchedulerInterface).

The proposal follows True Async's transparent-asynchrony model: a blocking I/O
operation inside a coroutine suspends that coroutine, while the same operation
outside a coroutine retains its ordinary blocking behavior. This lets libraries
use one callable API instead of maintaining separate synchronous and
asynchronous variants.

## Types

| Type                                                     | Description                                      |
| -------------------------------------------------------- | ------------------------------------------------ |
| [`Awaitable<T>`](thp:std.async.Awaitable)                | Marks a value whose readiness can be observed.   |
| [`Completable<T>`](thp:std.async.Completable)            | Defines a one-shot awaitable result.             |
| [`Coroutine<T>`](thp:std.async.Coroutine)                | Represents a scheduled callable and its result.  |
| [`Timeout`](thp:std.async.Timeout)                       | Represents a one-shot timer used to limit waits. |
| [`SchedulerInterface`](thp:std.async.SchedulerInterface) | Defines the scheduler integration contract.      |

## Functions

| Function                                                                   | Description                                          |
| -------------------------------------------------------------------------- | ---------------------------------------------------- |
| [`scheduler_register_default()`](thp:std.async.scheduler_register_default) | Registers THP's built-in scheduler.                  |
| [`scheduler_register()`](thp:std.async.scheduler_register)                 | Registers an application-provided scheduler.         |
| [`scheduler_is_registered()`](thp:std.async.scheduler_is_registered)       | Reports whether a scheduler has been registered.     |
| [`async()`](thp:std.async.async)                                           | Creates and schedules a coroutine.                   |
| [`await()`](thp:std.async.await)                                           | Waits for a one-shot asynchronous result.            |
| [`suspend()`](thp:std.async.suspend)                                       | Cooperatively yields the current coroutine.          |
| [`delay()`](thp:std.async.delay)                                           | Suspends the current coroutine for a duration.       |
| [`timeout()`](thp:std.async.timeout)                                       | Creates a timer that can cancel an outstanding wait. |

## Example

Start independent operations before awaiting either result:

```thp
scheduler_register_default();

$profile = async(loadProfile(...));
$settings = async(loadSettings(...));

render(
    await($profile, timeout(2_000)),
    await($settings, timeout(2_000)),
);
```

Both coroutines are scheduled before the first call to `await()`, so they can
make progress concurrently whenever either operation suspends. Each timeout
limits only its corresponding wait.

## Execution model

- Coroutines are concurrent, not parallel. One coroutine executes on the
  scheduler thread at a time.
- Scheduling is cooperative. A coroutine gives other work an opportunity to run
  when it suspends, awaits another coroutine, or performs supported non-blocking
  I/O.
- `async()` is a normal function, not a function modifier. THP does not divide
  the language into colored synchronous and asynchronous functions.
- Cancellation is cooperative. It is observed when a coroutine next suspends;
  it cannot interrupt arbitrary CPU-bound code.
- A long computation with no suspension point blocks every other coroutine on
  the same scheduler thread.

## Runtime integration boundary

This proposal does not import True Async's list of coroutine-aware PHP
functions as a THP compatibility promise. No I/O operation in this checkout is
implemented as an Async suspension point. Each THP runtime adapter must document
the operations it supports; calling an unsupported blocking operation inside a
coroutine blocks the scheduler thread.

## Design background

The coroutine lifecycle, transparent I/O suspension, one-shot completion
contract, and cooperative cancellation are adapted from
[True Async's documentation](https://true-async.github.io/en/docs.html). THP
adds generic result types and retains an explicit scheduler-registration API so
applications can choose the runtime before creating work.

This proposal covers the core execution model. Structured concurrency,
channels, futures, context propagation, and parallel worker APIs remain future
design work rather than implied parts of this contract.
