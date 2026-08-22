---
kind: method
id: std.spl.SplSubject::notify
title: SplSubject::notify
summary: Notifies every registered observer.
name: notify
order: 3
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
owner: std.spl.SplSubject
visibility: public
modifiers: []
---

[`SplSubject`](thp:std.spl.SplSubject)`::notify()` notifies every registered observer.

## Behavior

Notifies every registered observer.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->notify();
```

The call uses the signature and defaults documented above.

## See also

- [`SplSubject`](thp:std.spl.SplSubject)
