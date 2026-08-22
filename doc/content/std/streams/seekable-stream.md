---
kind: interface
id: std.streams.SeekableStream
title: SeekableStream
summary: Adds absolute cursor inspection and relative seeking.
name: SeekableStream
module: streams
typeParameters: []
interfaces:
  - id: std.streams.Stream
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

Seeking returns the new absolute byte position. Memory, temporary, and
regular-file streams allow positions beyond the current end; a later write fills
the gap with zero bytes.
