---
kind: method
id: std.async.SchedulerInterface::suspend
title: SchedulerInterface::suspend
summary:
  Suspends the currently running coroutine and returns control to the scheduler.
  This method does not return a value.
name: suspend
order: 1
typeParameters: []
parameters: []
returns:
  type: void
  description: This method does not return a value.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.async.SchedulerInterface
visibility: public
modifiers: []
---

[`SchedulerInterface`](thp:std.async.SchedulerInterface)`::suspend()` suspends the currently running coroutine and returns control to the scheduler. This method does not return a value.

## Behavior

Suspends the currently running coroutine and returns control to the scheduler. This method does not return a value.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->suspend();
```

The call uses the signature and defaults documented above.

## See also

- [`SchedulerInterface`](thp:std.async.SchedulerInterface)
