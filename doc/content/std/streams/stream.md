---
kind: interface
id: std.streams.Stream
title: Stream
summary: Provides shared lifetime state and runtime stream-capability inspection.
name: Stream
module: streams
typeParameters: []
interfaces:
  - id: std.streams.Closeable
constants: []
properties: []
status: experimental
availability: proposed
notice: This capability-inspection interface is proposed and is not implemented.
version: "0.1"
---

`Stream` is the common type returned by dynamic URI opening. Capability
interfaces expose reading, writing, and seeking only when those operations are
available.

All aliases share the same cursor and close state. Closing any alias invalidates
operations through every alias while repeated `close()` calls remain harmless.
