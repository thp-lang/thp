---
kind: method
id: std.spl.SplObserver::update
title: SplObserver::update
summary: Receives a subject notification.
name: update
order: 1
typeParameters: []
parameters:
  - name: subject
    type: SplSubject
    description: Value supplied as $subject.
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
owner: std.spl.SplObserver
visibility: public
modifiers: []
---

[`SplObserver`](thp:std.spl.SplObserver)`::update()` receives a subject notification.

## Behavior

Receives a subject notification.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->update($subject);
```

The call uses the signature and defaults documented above.

## See also

- [`SplObserver`](thp:std.spl.SplObserver)
