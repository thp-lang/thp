---
kind: class
id: std.async.WaitCancelledException
title: WaitCancelledException
summary: Reports that an await cancellation source won the wait.
name: WaitCancelledException
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

`WaitCancelledException` is thrown by `await($target, $cancellation)` when the
cancellation source is the first terminal notification processed. Its previous
exception is the source's stored failure if that source failed, or `null` if it
completed successfully. The target continues running.
