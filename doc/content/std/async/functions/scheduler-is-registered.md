---
kind: function
id: std.async.scheduler_is_registered
title: scheduler_is_registered
summary: Reports whether a coroutine scheduler has been registered.
name: scheduler_is_registered
order: 9
typeParameters: []
parameters: []
returns:
  type: bool
  description:
    true after either scheduler_register() or scheduler_register_default()
    completes successfully; otherwise false.
errors:
  - description: This function does not throw for an unregistered scheduler. It returns false.
related: []
status: experimental
availability: proposed
notice: This function is proposed and is not implemented in this repository. The
  scheduler lifecycle may change.
version: "0.1"
module: async
---

`scheduler_is_registered()` reports whether the process has an active scheduler
registration.

## Behavior

The function only inspects registration state. It does not create a scheduler,
start the scheduler, enqueue work, or suspend the current execution context.

A `true` result means a scheduler is available to [`async()`](thp:std.async.async) and
[`await()`](thp:std.async.await). It does not indicate whether the scheduler currently has
pending work.

## Example

Library or framework bootstrap code can preserve an application-provided
scheduler and install the default only when necessary:

```thp
if (!scheduler_is_registered()) {
    scheduler_register_default();
}
```

The check and registration are separate operations. Perform this pattern during
single-threaded application startup; it is not an atomic guard for competing
registration attempts.

## See also

- [`scheduler_register()`](thp:std.async.scheduler_register)
- [`scheduler_register_default()`](thp:std.async.scheduler_register_default)
- [`SchedulerInterface`](thp:std.async.SchedulerInterface)
- [`async()`](thp:std.async.async)
- [`await()`](thp:std.async.await)
- [`delay()`](thp:std.async.delay)
- [`timeout()`](thp:std.async.timeout)
