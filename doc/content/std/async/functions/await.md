---
kind: function
id: std.async.await
title: await
summary: Waits for a coroutine and returns its result.
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
  - description:
      If $awaitable failed, await() rethrows its stored failure in the awaiting
      context. A cancelled $awaitable produces the Async cancellation failure.
      Completion of $cancellation produces a distinct wait-cancelled failure, with the
      source failure available as its cause. Concrete failure types are not yet
      finalized.
  - description:
      The call also fails if pending work cannot be driven because no scheduler
      is registered.
related: []
status: experimental
availability: proposed
notice:
  await() is proposed and is not implemented in this repository. Cancellation and
  concrete error types are not finalized.
version: "0.1"
module: async
---

`await()` waits for any one-shot asynchronous operation and returns its typed
result.

## Behavior

If `$awaitable` is still pending, `await()` suspends the current coroutine so
the scheduler can run other work. Execution resumes when `$awaitable`
completes. Awaiting an already completed operation returns its stored result
immediately.

At top level, where there is no calling coroutine to suspend, `await()` gives
control to the registered scheduler until the requested operation completes.

A completed operation may be awaited more than once. Each call observes the
same result or failure.

If `$cancellation` completes first, the current call stops waiting. This does
not automatically cancel `$awaitable`; call its `cancel()` method when the
underlying operation should also be asked to stop.

## Example

```thp
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
