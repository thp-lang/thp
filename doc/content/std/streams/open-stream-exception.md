---
kind: class
id: std.streams.OpenStreamException
title: OpenStreamException
summary: Reports failure to open a path or stream URI.
name: OpenStreamException
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

`OpenStreamException` retains the requested path or URI and a platform
error code when one is available. A zero system code means no platform code was
reported.
