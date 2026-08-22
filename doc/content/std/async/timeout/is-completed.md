---
kind: method
id: std.async.Timeout::isCompleted
title: Timeout::isCompleted
summary: Returns true after the timer fires or is cancelled.
name: isCompleted
order: 2
typeParameters: []
parameters: []
returns:
  type: bool
  description: Returns true after the timer fires or is cancelled.
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

[`Timeout`](thp:std.async.Timeout)`::isCompleted()` returns true after the timer fires or is cancelled.

## Behavior

Returns true after the timer fires or is cancelled.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isCompleted();
```

The call uses the signature and defaults documented above.

## See also

- [`Timeout`](thp:std.async.Timeout)
