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
availability: implemented
notice: The VM produces this exception for operations through a closed stream alias.
version: "0.1"
---

Every alias observes one shared close state. Operations requiring an open
handle throw `ClosedStreamException` after any alias closes it; closing again is
idempotent and does not throw.
