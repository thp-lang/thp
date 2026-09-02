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
availability: implemented
notice: The VM produces this exception for unsupported executable URI mode combinations.
version: "0.1"
---

Typed factories prevent this failure by returning capability-specific
types. A dynamic `Stream` must be narrowed before read, write, or seek operations
are called.
