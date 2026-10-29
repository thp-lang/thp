---
kind: function
id: std.async.await
title: await
summary: Waits for a one-shot completion and returns its result.
name: await
order: 11
typeParameters:
  - name: T
    description: The T type parameter.
parameters:
  - name: awaitable
    type: Completable<T>
    description: The one-shot operation whose result is needed.
  - name: cancellation
    type: ?Completable<mixed>
    description: An optional source that stops this wait on completion.
    default: "null"
returns:
  type: T
  description:
    The operation's successful value of type T. For a Coroutine, this is the
    value returned by the callable passed to async().
errors:
  - description: A failed target rethrows its stored Throwable; cancellation of
      the target throws CancellationException.
  - description: If the cancellation source wins, WaitCancelledException carries
      its stored failure as the cause when it failed and leaves the target running.
  - description: A pending target without registration throws Error at this call site.
related: []
status: experimental
availability: implemented
version: "0.1"
module: async
---

`await()` waits for any one-shot asynchronous operation and returns its typed
result.

## Behavior

If `$awaitable` is still pending, `await()` requires registration, subscribes
to the target and optional cancellation source, and suspends the current
coroutine. The first terminal notification processed wins. THP removes the
losing subscription before resuming. An already completed result can be read
without registration.

After registration the main script is the root coroutine; its `await()` yields
to the scheduler just like a spawned coroutine.

A completed operation may be awaited more than once. Each call observes the
same result or failure.

If `$cancellation` completes first, this call throws `WaitCancelledException`
with the source failure as its cause. It does not cancel `$awaitable`.
The winning source's failure counts as observed for request-end error reporting.

## Example

```thp
<?thp

scheduler_register_default();

$answer = async(calculateAnswer(...));

echo await($answer, timeout(500));
```

### Failure propagation

```thp
$operation = async(function (): string {
    throw new RuntimeException("Request failed");
});

try {
    await($operation);
} catch (RuntimeException $error) {
    report($error);
}
```

The failure is captured when the coroutine runs and rethrown by `await()`.

## See also

- [`Completable<T>`](thp:std.async.Completable)
- [`Coroutine<T>`](thp:std.async.Coroutine)
- [`async()`](thp:std.async.async)
- [`timeout()`](thp:std.async.timeout)
- [`scheduler_register_default()`](thp:std.async.scheduler_register_default)
- [`scheduler_register()`](thp:std.async.scheduler_register)
- [`scheduler_is_registered()`](thp:std.async.scheduler_is_registered)
- [True Async `await()`](https://true-async.github.io/en/docs/reference/await.html)
