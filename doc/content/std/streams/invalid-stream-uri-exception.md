---
kind: class
id: std.streams.InvalidStreamUriException
title: InvalidStreamUriException
summary: Reports an unknown or malformed stream URI.
name: InvalidStreamUriException
module: streams
typeParameters: []
parent:
  id: std.streams.OpenStreamException
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

This exception is raised before I/O begins, so its system error code is zero.
Unknown schemes and malformed wrapper options are invalid URIs.
