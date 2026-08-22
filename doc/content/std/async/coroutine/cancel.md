---
kind: method
id: std.async.Coroutine::cancel
title: Coroutine::cancel
summary:
  Requests cancellation. A running coroutine receives the cancellation failure at
  its next suspension point, and its finally blocks still execute. Calling cancel()
  after terminal completion has no effect.
name: cancel
order: 8
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
owner: std.async.Coroutine
visibility: public
modifiers: []
---

[`Coroutine`](thp:std.async.Coroutine)`::cancel()` requests cancellation. A running coroutine receives the cancellation failure at its next suspension point, and its finally blocks still execute. Calling cancel() after terminal completion has no effect.

## Behavior

Requests cancellation. A running coroutine receives the cancellation failure at its next suspension point, and its finally blocks still execute. Calling cancel() after terminal completion has no effect.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->cancel();
```

The call uses the signature and defaults documented above.

## See also

- [`Coroutine`](thp:std.async.Coroutine)
