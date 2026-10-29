---
kind: class
id: std.async.CancellationException
title: CancellationException
summary: Reports an uncaught coroutine cancellation checkpoint.
name: CancellationException
module: async
typeParameters: []
parent:
  id: std.spl.RuntimeException
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
version: "0.1"
---

`CancellationException` is delivered at the next suspension checkpoint after
`Coroutine::cancel()` requests cancellation. A queued coroutine is cancelled
before its body starts. A coroutine that catches this exception and returns
completes successfully.
