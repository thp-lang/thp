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
  - description: "The call fails when:"
  - description: a scheduler has already been registered;
  - description: asynchronous work started before registration completed; or
  - description: the built-in scheduler cannot be initialized.
  - description: The concrete error types are not yet finalized.
related: []
status: experimental
availability: proposed
notice: This function is proposed and is not implemented in this repository. The
  built-in scheduler and its registration lifecycle may change.
version: "0.1"
module: async
---

`scheduler_register_default()` creates and activates THP's built-in scheduler.
It is the recommended way to enable asynchronous execution when an application
does not provide its own scheduler.

## Behavior

Registration is process-wide and must happen before the first call to
[`async()`](thp:std.async.async) or [`await()`](thp:std.async.await). The built-in scheduler remains
active for the lifetime of the process.

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
