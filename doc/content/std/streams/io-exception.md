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
availability: implemented
notice: The VM produces this typed exception hierarchy for executable stream failures.
version: "0.1"
---

`IoException` reports failures while opening, reading, writing, flushing,
seeking, or closing an I/O handle. Invalid arguments rejected before I/O use
`ValueError` instead.
