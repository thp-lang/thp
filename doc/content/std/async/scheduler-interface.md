---
kind: interface
id: std.async.SchedulerInterface
title: SchedulerInterface
summary: Selects the next ready coroutine from THP's request-owned queue.
name: SchedulerInterface
module: async
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
version: "0.1"
---

`SchedulerInterface` has one method, [`chooseNext()`](thp:std.async.SchedulerInterface::chooseNext):

```thp
interface SchedulerInterface {
    function chooseNext(vector<int> $ready): int;
}
```

## Contract

THP owns the ready queue, timers, I/O readiness, cancellation, and VM context
switching. `chooseNext()` receives the IDs currently ready and must return one
of those IDs. IDs are unique within a request. The method cannot suspend or
reenter the scheduler. An invalid choice throws `Error`; an exception from the
callback fails the request.

The default scheduler chooses the first ready ID in FIFO order. A custom
scheduler changes selection only; it cannot create or run coroutines directly.

## Example

The following scheduler selects the last ready coroutine:

```thp
<?thp

final class LastReadyScheduler implements SchedulerInterface {
    function chooseNext(vector<int> $ready): int {
        return $ready[count($ready) - 1];
    }
}
```

## See also

- [`scheduler_register()`](thp:std.async.scheduler_register)
- [`scheduler_register_default()`](thp:std.async.scheduler_register_default)
- [`scheduler_is_registered()`](thp:std.async.scheduler_is_registered)
- [`async()`](thp:std.async.async)
- [`await()`](thp:std.async.await)
- [`suspend()`](thp:std.async.suspend)
- [`delay()`](thp:std.async.delay)
- [True Async scheduler hook RFC](https://true-async.github.io/en/rfc.html)
