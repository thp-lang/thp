---
kind: class
id: std.streams.UnsupportedStreamOperationException
title: UnsupportedStreamOperationException
summary: Reports a capability unavailable on a dynamically opened stream.
name: UnsupportedStreamOperationException
module: streams
typeParameters: []
parent:
  id: std.streams.IoException
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

Typed factories prevent this failure by returning capability-specific
types. A dynamic `Stream` must be narrowed before read, write, or seek operations
are called.
