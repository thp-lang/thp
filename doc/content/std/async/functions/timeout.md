---
kind: function
id: std.async.timeout
title: timeout
summary: Creates a one-shot timer that can limit an asynchronous wait.
name: timeout
order: 14
typeParameters: []
parameters:
  - name: milliseconds
    type: int
    description: Positive timeout duration, in milliseconds.
returns:
  type: Timeout
  description: A new Timeout registered with the active scheduler.
errors:
  - description:
      A non-positive $milliseconds value is invalid. The call also fails when no
      scheduler is registered or the active scheduler cannot allocate a timer. Concrete
      error types are not yet finalized.
related: []
status: experimental
availability: proposed
notice:
  timeout() is proposed and is not implemented in this repository. Its concrete
  failure type is not finalized.
version: "0.1"
module: async
---

`timeout()` creates a timer that completes with a timeout failure after the
requested duration.

## Behavior

The timeout becomes ready after at least `$milliseconds` milliseconds. It is
normally supplied as the optional cancellation source to [`await()`](thp:std.async.await).
If the timeout completes first, `await()` stops waiting but leaves the target
operation running.

Cancel a timeout that is no longer needed so the scheduler can release its timer
resources promptly.

## Example

```thp
scheduler_register_default();

$request = async(fetchText(...), "/slow");

try {
    echo await($request, timeout(1_000));
} catch (Throwable $failure) {
    $request->cancel();
    report($failure);
}
```

The explicit `cancel()` asks the underlying request to stop after the wait times
out. Catching `Throwable` is temporary while the timeout failure class remains
unsettled.

## See also

- [`Timeout`](thp:std.async.Timeout)
- [`await()`](thp:std.async.await)
- [`delay()`](thp:std.async.delay)
- [`Completable<T>`](thp:std.async.Completable)
- [True Async `timeout()`](https://true-async.github.io/en/docs/reference/timeout.html)
