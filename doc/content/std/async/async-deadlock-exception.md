---
kind: class
id: std.async.AsyncDeadlockException
title: AsyncDeadlockException
summary: Reports pending work with no possible wakeup.
name: AsyncDeadlockException
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

THP throws `AsyncDeadlockException` when work is pending but nothing can run
and no timer or I/O operation can wake it. A hard request limit retains its
existing policy-failure behavior.
