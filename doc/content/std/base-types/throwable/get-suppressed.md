---
kind: method
id: std.baseTypes.Throwable::getSuppressed
title: Throwable::getSuppressed
summary:
  Returns cleanup failures retained while this throwable was already propagating.
  The returned vector is empty when no failure was suppressed.
name: getSuppressed
order: 8
typeParameters: []
parameters: []
returns:
  type: vector<Throwable>
  description:
    Returns cleanup failures retained while this throwable was already
    propagating. The returned vector is empty when no failure was suppressed.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: partial
notice:
  The compiler and reference VM implement this member. Suppressed failures are
  currently produced by `using` cleanup.
version: "0.1"
owner: std.baseTypes.Throwable
visibility: public
modifiers: []
---

[`Throwable`](thp:std.baseTypes.Throwable)`::getSuppressed()` returns cleanup failures retained while this throwable was already propagating. The returned vector is empty when no failure was suppressed.

## Behavior

Returns cleanup failures retained while this throwable was already propagating. The returned vector is empty when no failure was suppressed.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = (new Exception())->getSuppressed();
```

The call uses the signature and defaults documented above.

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)
