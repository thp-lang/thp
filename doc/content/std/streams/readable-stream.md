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
availability: implemented
notice: Reading executes on memory, temporary, readable-file, read-write-file, and request-input streams.
version: "0.1"
---

Reads operate on arbitrary byte strings without UTF-8 validation or
transcoding. `read(0)` returns an empty string, while `eof()` distinguishes an
empty request from end of stream.

`readAll()` preserves the cursor when a non-null byte limit would be exceeded.
