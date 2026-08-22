---
kind: method
id: std.async.Coroutine::isStarted
title: Coroutine::isStarted
summary: Returns true after the coroutine begins its first scheduling turn.
name: isStarted
order: 2
typeParameters: []
parameters: []
returns:
  type: bool
  description: Returns true after the coroutine begins its first scheduling turn.
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

[`Coroutine`](thp:std.async.Coroutine)`::isStarted()` returns true after the coroutine begins its first scheduling turn.

## Behavior

Returns true after the coroutine begins its first scheduling turn.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isStarted();
```

The call uses the signature and defaults documented above.

## See also

- [`Coroutine`](thp:std.async.Coroutine)
