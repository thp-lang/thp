---
kind: function
id: std.async.suspend
title: suspend
summary: Cooperatively yields execution of the current coroutine.
name: suspend
order: 12
typeParameters: []
parameters: []
returns:
  type: void
  description:
    This function does not return a value. The current callable continues after
    the scheduler resumes its coroutine.
errors:
  - description: Invalid context throws Error at this call site.
  - description: Pending cancellation throws CancellationException at this checkpoint.
related: []
status: experimental
availability: implemented
version: "0.1"
module: async
---

`suspend()` yields the current coroutine so the scheduler can run other ready
work.

## Behavior

The current coroutine yields and is requeued at the back of the ready queue.
The scheduler makes it eligible to run again on a later turn.
No parallel execution is introduced.

`suspend()` is a cancellation checkpoint. If cancellation has been requested,
the coroutine receives `CancellationException` instead of continuing
past the call.

Supported I/O functions suspend automatically while waiting, so application
code normally calls `suspend()` only to provide fairness during CPU-side loops
or to integrate a low-level asynchronous source.

## Example

```thp
scheduler_register_default();

$first = async(function (): void {
    echo "A";
    suspend();
    echo "C";
});

$second = async(function (): void {
    echo "B";
});

await($first);
await($second);
```

The first coroutine yields after printing `A`, allowing the second coroutine to
print `B` before the first continues with `C`.

## See also

- [`delay()`](thp:std.async.delay)
- [`async()`](thp:std.async.async)
- [`Coroutine<T>`](thp:std.async.Coroutine)
- [`SchedulerInterface`](thp:std.async.SchedulerInterface)
- [True Async Coroutine suspension](https://true-async.github.io/en/docs/components/coroutines.html#suspension-suspend)
