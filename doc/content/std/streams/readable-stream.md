---
kind: interface
id: std.streams.ReadableStream
title: ReadableStream
summary: Adds byte-oriented reading and end-of-stream inspection.
name: ReadableStream
module: streams
typeParameters: []
interfaces:
  - id: std.streams.Stream
constants: []
properties: []
status: experimental
availability: partial
notice:
  read(), readAll(), eof(), tell(), and closing execute. The documented Stream
  parent and capability-inspection methods remain proposed.
version: "0.1"
---

Reads operate on arbitrary byte strings without UTF-8 validation or
transcoding. `read(0)` returns an empty string, while `eof()` distinguishes an
empty request from end of stream.

`readAll()` preserves the cursor when a non-null byte limit would be exceeded.
