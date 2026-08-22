---
kind: method
id: std.async.Timeout::cancel
title: Timeout::cancel
summary:
  Cancels a pending timer and releases its scheduler resources. Calling the
  method after the timer has fired has no effect.
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
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.async.Timeout
visibility: public
modifiers: []
---

[`Timeout`](thp:std.async.Timeout)`::cancel()` cancels a pending timer and releases its scheduler resources. Calling the method after the timer has fired has no effect.

## Behavior

Cancels a pending timer and releases its scheduler resources. Calling the method after the timer has fired has no effect.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->cancel();
```

The call uses the signature and defaults documented above.

## See also

- [`Timeout`](thp:std.async.Timeout)
