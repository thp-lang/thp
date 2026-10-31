---
kind: method
id: std.async.Completable::result
title: Completable::result
summary: Reads a completed value or rethrows its stored failure.
name: result
order: 5
typeParameters: []
parameters: []
returns:
  type: T
  description: The stored successful value.
errors:
  - description:
      Pending completion throws Error; a failed or cancelled completion
      rethrows its stored Throwable.
related: []
status: experimental
availability: implemented
version: "0.1"
owner: std.async.Completable
visibility: public
modifiers: []
---

`result()` never suspends. It returns the same value on repeated reads after
success, throws the stored failure after failure or cancellation, and throws
`Error` when called while pending.
