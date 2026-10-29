---
kind: method
id: std.async.Completable::subscribe
title: Completable::subscribe
summary: Registers a one-shot terminal-completion notification.
name: subscribe
order: 4
typeParameters: []
parameters:
  - name: notify
    type: callable<void>
    description: Callback invoked when the operation reaches a terminal state.
returns:
  type: callable<void>
  description: An idempotent callable that removes the subscription.
errors:
  - description:
      The notification callback cannot suspend or reenter the scheduler;
      a callback failure fails the request.
related: []
status: experimental
availability: implemented
version: "0.1"
owner: std.async.Completable
visibility: public
modifiers: []
---

`subscribe()` calls `$notify` exactly once when this operation completes,
fails, or is cancelled. If it is already terminal, notification happens before
`subscribe()` returns. The returned callable removes a pending notification;
calling it again has no effect. The notification and unsubscribe callables
cannot suspend. A notification callback that throws fails the request.
