---
kind: method
id: std.async.SchedulerInterface::delay
title: SchedulerInterface::delay
summary:
  Suspends the current coroutine and arranges for it to become ready after at
  least $milliseconds milliseconds. A zero duration requeues it without a timer.
name: delay
order: 2
typeParameters: []
parameters:
  - name: milliseconds
    type: int
    description: Value supplied as $milliseconds.
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

[`SchedulerInterface`](thp:std.async.SchedulerInterface)`::delay()` suspends the current coroutine and arranges for it to become ready after at least $milliseconds milliseconds. A zero duration requeues it without a timer.

## Behavior

Suspends the current coroutine and arranges for it to become ready after at least $milliseconds milliseconds. A zero duration requeues it without a timer.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->delay($milliseconds);
```

The call uses the signature and defaults documented above.

## See also

- [`SchedulerInterface`](thp:std.async.SchedulerInterface)
