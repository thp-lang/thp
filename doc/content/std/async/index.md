---
kind: module
id: std.async
title: Async
summary: Concurrency primitives and cooperatively scheduled operations.
module: async
order: 10
status: experimental
availability: implemented
---

THP's Async API runs ordinary callables in cooperatively scheduled coroutines.
A scheduler must be registered before starting asynchronous work.
Applications can select the built-in scheduler or provide an implementation of
[`SchedulerInterface`](thp:std.async.SchedulerInterface).

Memory-stream operations complete immediately. Direct file-backed opens,
reads, writes, seeks, EOF checks, flushes, `SplFileObject` line methods,
metadata queries, directory construction and child discovery, and temporary-file spill suspend
while a standard-library worker pool performs host I/O. The
pool has four process-wide workers and a 64-job queue; operations on aliased
handles are serialized per stream to preserve cursor order. Native iterator
adapters resume callbacks and file operations through coroutine frames.
`thp:/input` remains usable, but its synchronous host reader can block the
request thread. Host output and ABI-v1 native callbacks can also block all
coroutines.

## Types

| Type                                                                                                                                                                                                                                                     | Description                                      |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| [`Awaitable<T>`](thp:std.async.Awaitable)                                                                                                                                                                                                                | Marks a value whose readiness can be observed.   |
| [`Completable<T>`](thp:std.async.Completable)                                                                                                                                                                                                            | Defines a one-shot awaitable result.             |
| [`Coroutine<T>`](thp:std.async.Coroutine)                                                                                                                                                                                                                | Represents a scheduled callable and its result.  |
| [`Timeout`](thp:std.async.Timeout)                                                                                                                                                                                                                       | Represents a one-shot timer used to limit waits. |
| [`SchedulerInterface`](thp:std.async.SchedulerInterface)                                                                                                                                                                                                 | Defines the scheduler integration contract.      |
| [`CancellationException`](thp:std.async.CancellationException), [`WaitCancelledException`](thp:std.async.WaitCancelledException), [`TimeoutException`](thp:std.async.TimeoutException), [`AsyncDeadlockException`](thp:std.async.AsyncDeadlockException) | Report the four Async failures.                  |

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
- `async()` and `await()` are ordinary function calls. There is no `async`
  function modifier or `await` expression syntax. Incompatible arguments to
  `async()` are compile errors at their argument spans; invalid API use throws
  `Error` at the call site.
- Cancellation is cooperative. It is observed when a coroutine next suspends;
  it cannot interrupt arbitrary CPU-bound code.
- A long computation with no suspension point blocks every other coroutine on
  the same scheduler thread.
- Scheduler and completion callbacks cannot suspend. The runtime owns the ready
  queue, timers, I/O readiness, cancellation, and VM switching.
- Registration, coroutine IDs, and pending work belong to one request. The main
  script is the root coroutine after registration. THP drains spawned work when
  the main script ends, including after an uncaught exception.
- No runnable work and no possible timer or I/O wakeup is a deadlock. At drain, an
  unobserved failed coroutine fails the request; the earliest coroutine ID is
  primary and later failures are suppressed.

## Runtime integration boundary

This contract does not import True Async's coroutine-aware PHP functions as a
THP compatibility promise. An issued OS file write can finish after its
coroutine is cancelled. Hard request limits keep their existing policy-failure
behavior; instruction and heap budgets cover every coroutine in the request.
The public C ABI remains version 1.

## Design background

The coroutine lifecycle, transparent I/O suspension, one-shot completion
contract, and cooperative cancellation are adapted from
[True Async's documentation](https://true-async.github.io/en/docs.html). THP
adds generic result types and retains an explicit scheduler-registration API so
applications can choose the runtime before creating work.

This contract covers the core execution model. Structured concurrency,
channels, futures, context propagation, and parallel worker APIs remain future
design work rather than implied parts of this contract.
