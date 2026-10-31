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
  - description: A second registration throws Error at this call site.
related: []
status: experimental
availability: implemented
version: "0.1"
module: async
---

`scheduler_register()` activates an application-provided scheduler for the
current request.

## Behavior

Registration starts false in every request and succeeds once per request.
After registration THP owns coroutine creation, the ready queue, timers,
I/O readiness, cancellation, and VM switching. It asks `$scheduler` only to
select an ID from the current ready set.

Register before creating coroutines. A second call to either registration
function throws `Error`. No scheduler or coroutine state crosses requests.

## Example

```thp
final class ApplicationScheduler implements SchedulerInterface {
    function chooseNext(vector<int> $ready): int {
        return $ready[0];
    }
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
