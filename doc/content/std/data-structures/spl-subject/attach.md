---
kind: method
id: std.spl.SplSubject::attach
title: SplSubject::attach
summary: Registers an observer.
name: attach
order: 1
typeParameters: []
parameters:
  - name: observer
    type: SplObserver
    description: Value supplied as $observer.
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

[`SplSubject`](thp:std.spl.SplSubject)`::attach()` registers an observer.

## Behavior

Registers an observer.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->attach($observer);
```

The call uses the signature and defaults documented above.

## See also

- [`SplSubject`](thp:std.spl.SplSubject)
