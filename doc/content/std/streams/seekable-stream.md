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
availability: implemented
notice: Absolute, relative, and end-origin seeking execute for memory, temporary, and regular-file streams.
version: "0.1"
---

Seeking returns the new absolute byte position. Memory, temporary, and
regular-file streams allow positions beyond the current end; a later write fills
the gap with zero bytes.
