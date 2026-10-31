---
kind: method
id: std.async.Coroutine::getId
title: Coroutine::getId
summary: Returns an identifier unique among coroutines in the current request.
name: getId
order: 1
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns an identifier unique among coroutines in the current request.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
version: "0.1"
owner: std.async.Coroutine
visibility: public
modifiers: []
---

[`Coroutine`](thp:std.async.Coroutine)`::getId()` returns an identifier unique among coroutines in the current request.

## Behavior

Returns an identifier unique among coroutines in the current request.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getId();
```

The call uses the signature and defaults documented above.

## See also

- [`Coroutine`](thp:std.async.Coroutine)
