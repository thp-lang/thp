---
kind: function
id: std.async.delay
title: delay
summary: Suspends the current coroutine for a number of milliseconds.
name: delay
order: 13
typeParameters: []
parameters:
  - name: milliseconds
    type: int
    description: Non-negative suspension duration, in milliseconds.
returns:
  type: void
  description: This function does not return a value.
errors:
  - description:
      A negative $milliseconds value is invalid. The call also fails when no
      scheduler is registered or the active scheduler cannot create the timer. Concrete
      error types are not yet finalized.
related: []
status: experimental
availability: proposed
notice:
  delay() is proposed and is not implemented in this repository. Timing precision
  and cancellation failure types are not finalized.
version: "0.1"
module: async
---

`delay()` suspends the current coroutine until at least the requested duration
has elapsed.

## Behavior

For a positive duration, the scheduler arranges a timer and runs other ready
work. The coroutine becomes eligible to resume after the duration has elapsed;
load may make the actual delay longer.

`delay(0)` yields and requeues the current coroutine without arranging a timer.
Use it as a fairness point in a loop. `delay()` does not block the scheduler
thread.

The call is a cancellation checkpoint. A pending cancellation request is
observed before normal execution resumes.

## Example

```thp
scheduler_register_default();

$heartbeat = async(function (): string {
    delay(250);
    return "ready";
});

echo await($heartbeat);
```

Other ready coroutines may run during the 250-millisecond delay.

## See also

- [`suspend()`](thp:std.async.suspend)
- [`timeout()`](thp:std.async.timeout)
- [`Coroutine<T>`](thp:std.async.Coroutine)
- [True Async `delay()`](https://true-async.github.io/en/docs/reference/delay.html)
