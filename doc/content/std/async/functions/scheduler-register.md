---
kind: function
id: std.async.scheduler_register
title: scheduler_register
summary: Registers an application-provided coroutine scheduler.
name: scheduler_register
order: 8
typeParameters: []
parameters:
  - name: scheduler
    type: SchedulerInterface
    description: The scheduler to make active.
returns:
  type: void
  description: This function does not return a value.
errors:
  - description: "The call fails when:"
  - description: a scheduler has already been registered;
  - description: asynchronous work started before registration completed; or
  - description: the runtime cannot activate the supplied scheduler.
  - description: The concrete error types are not yet finalized.
related: []
status: experimental
availability: proposed
notice: This function is proposed and is not implemented in this repository. The
  registration lifecycle and concrete error types are not finalized.
version: "0.1"
module: async
---

`scheduler_register()` activates an application-provided scheduler for
asynchronous work in the current process.

## Behavior

Registration is process-wide. After the function returns, [`async()`](thp:std.async.async)
uses `$scheduler` to create and enqueue new coroutines, and [`await()`](thp:std.async.await)
uses it when the current execution context must suspend.

Register the scheduler once, during application startup, before creating any
coroutines. The registered scheduler remains active for the lifetime of the
process.

## Example

```thp
final class ApplicationScheduler implements SchedulerInterface
{

    // SchedulerInterface methods omitted.
}

scheduler_register(new ApplicationScheduler());

$coroutine = async(function (): string {
    return loadMessage();
});

echo await($coroutine);
```

Use [`scheduler_register_default()`](thp:std.async.scheduler_register_default) when the
application does not need a custom scheduler.

## See also

- [`SchedulerInterface`](thp:std.async.SchedulerInterface)
- [`scheduler_register_default()`](thp:std.async.scheduler_register_default)
- [`scheduler_is_registered()`](thp:std.async.scheduler_is_registered)
- [`async()`](thp:std.async.async)
- [`await()`](thp:std.async.await)
- [`delay()`](thp:std.async.delay)
- [`timeout()`](thp:std.async.timeout)
