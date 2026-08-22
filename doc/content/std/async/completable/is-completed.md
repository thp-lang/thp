---
kind: method
id: std.async.Completable::isCompleted
title: Completable::isCompleted
summary: Returns true after successful, failed, or cancelled completion.
name: isCompleted
order: 2
typeParameters: []
parameters: []
returns:
  type: bool
  description: Returns true after successful, failed, or cancelled completion.
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
owner: std.async.Completable
visibility: public
modifiers: []
---

[`Completable`](thp:std.async.Completable)`::isCompleted()` returns true after successful, failed, or cancelled completion.

## Behavior

Returns true after successful, failed, or cancelled completion.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isCompleted();
```

The call uses the signature and defaults documented above.

## See also

- [`Completable`](thp:std.async.Completable)
