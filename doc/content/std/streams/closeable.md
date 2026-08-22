---
kind: interface
id: std.streams.Closeable
title: Closeable
summary: Defines deterministic, idempotent cleanup for a native handle or application object.
name: Closeable
module: streams
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
---

`Closeable` is the contract accepted by the language's `using` statement.
Its `close()` operation runs exactly once when control leaves the block, including
through `return`, loop control, or an exception.

An escaped alias remains valid as a value but observes the closed handle state.
If both the block body and cleanup fail, the cleanup failure is attached to the
body failure as a suppressed exception.
