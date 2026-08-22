---
kind: method
id: std.async.Coroutine::isRunning
title: Coroutine::isRunning
summary: Returns true while the coroutine is currently executing.
name: isRunning
order: 4
typeParameters: []
parameters: []
returns:
  type: bool
  description: Returns true while the coroutine is currently executing.
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

[`Coroutine`](thp:std.async.Coroutine)`::isRunning()` returns true while the coroutine is currently executing.

## Behavior

Returns true while the coroutine is currently executing.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isRunning();
```

The call uses the signature and defaults documented above.

## See also

- [`Coroutine`](thp:std.async.Coroutine)
