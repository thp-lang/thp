---
kind: class
id: std.streams.ClosedStreamException
title: ClosedStreamException
summary: Reports an operation attempted through a closed stream alias.
name: ClosedStreamException
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

Every alias observes one shared close state. Operations requiring an open
handle throw `ClosedStreamException` after any alias closes it; closing again is
idempotent and does not throw.
