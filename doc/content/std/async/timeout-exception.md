---
kind: class
id: std.async.TimeoutException
title: TimeoutException
summary: Reports that a Timeout reached its monotonic deadline.
name: TimeoutException
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

`TimeoutException` is the stored failure of a fired `Timeout`. If the timeout
is used as an `await()` cancellation source, `await()` throws
`WaitCancelledException` and retains this failure as its cause.
