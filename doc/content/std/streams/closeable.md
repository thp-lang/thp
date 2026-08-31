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
availability: implemented
notice: The compiler and VM implement this interface and deterministic cleanup contract.
version: "0.1"
---

`Closeable` is the contract accepted by the language's `using` statement.
Its `close()` operation runs exactly once when control leaves the block, including
through `return`, loop control, or an exception.

An escaped alias remains valid as a value but observes the closed handle state.
If both the block body and cleanup fail, the cleanup failure is attached to the
body failure as a suppressed exception.
