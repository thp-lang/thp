---
kind: function
id: std.async.scheduler_register_default
title: scheduler_register_default
summary: Registers THP's built-in coroutine scheduler.
name: scheduler_register_default
order: 7
typeParameters: []
parameters: []
returns:
  type: void
  description: This function does not return a value.
errors:
  - description: A second registration throws Error at this call site.
related: []
status: experimental
availability: implemented
version: "0.1"
module: async
---

`scheduler_register_default()` creates and activates THP's built-in scheduler.
It is the recommended way to enable asynchronous execution when an application
does not provide its own scheduler.

## Behavior

Registration is request-local and must happen before starting asynchronous work.
The default scheduler selects ready coroutines in FIFO order. A second call to
either registration function throws `Error`.

Calling this function is equivalent to creating THP's default
[`SchedulerInterface`](thp:std.async.SchedulerInterface) implementation and registering
it with [`scheduler_register()`](thp:std.async.scheduler_register).

## Example

```thp
scheduler_register_default();

$coroutine = async(function (): string {
    return loadMessage();
});

echo await($coroutine);
```

## See also

- [`scheduler_register()`](thp:std.async.scheduler_register)
- [`scheduler_is_registered()`](thp:std.async.scheduler_is_registered)
- [`SchedulerInterface`](thp:std.async.SchedulerInterface)
- [`async()`](thp:std.async.async)
- [`await()`](thp:std.async.await)
- [`delay()`](thp:std.async.delay)
- [`timeout()`](thp:std.async.timeout)
