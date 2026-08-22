---
kind: method
id: std.async.Timeout::isCancelled
title: Timeout::isCancelled
summary: Returns true when cancellation occurred before the timer fired.
name: isCancelled
order: 3
typeParameters: []
parameters: []
returns:
  type: bool
  description: Returns true when cancellation occurred before the timer fired.
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

[`Timeout`](thp:std.async.Timeout)`::isCancelled()` returns true when cancellation occurred before the timer fired.

## Behavior

Returns true when cancellation occurred before the timer fired.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isCancelled();
```

The call uses the signature and defaults documented above.

## See also

- [`Timeout`](thp:std.async.Timeout)
