---
kind: method
id: std.async.Completable::cancel
title: Completable::cancel
summary: Requests cancellation. Calling it after completion has no effect.
name: cancel
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
availability: implemented
version: "0.1"
owner: std.async.Completable
visibility: public
modifiers: []
---

[`Completable`](thp:std.async.Completable)`::cancel()` requests cancellation. Calling it after completion has no effect.

## Behavior

Requests cancellation without suspending. Calling it after completion has no
effect. Each implementation defines how its pending operation reaches a
terminal state.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->cancel();
```

The call uses the signature and defaults documented above.

## See also

- [`Completable`](thp:std.async.Completable)
