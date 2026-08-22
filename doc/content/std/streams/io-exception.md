---
kind: class
id: std.streams.IoException
title: IoException
summary: Base exception for stream and other input/output failures.
name: IoException
module: streams
typeParameters: []
parent:
  id: std.baseTypes.Exception
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

`IoException` reports failures while opening, reading, writing, flushing,
seeking, or closing an I/O handle. Invalid arguments rejected before I/O use
`ValueError` instead.
