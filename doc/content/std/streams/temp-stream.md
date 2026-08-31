---
kind: class
id: std.streams.TempStream
title: TempStream
summary: Spills a readable, writable, seekable byte stream from memory to a temporary file.
name: TempStream
module: streams
typeParameters: []
interfaces:
  - id: std.streams.ReadableStream
  - id: std.streams.WritableStream
  - id: std.streams.SeekableStream
constants: []
properties: []
status: experimental
availability: partial
notice:
  Opening, spilling, reading, writeAll(), absolute seek(int), tell(), and closing
  execute. write(), flush(), and relative seeking remain proposed.
version: "0.1"
---

This is a final native stream class created through `TempStream::open()`.
It behaves like `MemoryStream` until a write would exceed the configured memory
threshold, then copies its contents once to an anonymous temporary file while
preserving the cursor.

The selected backend is intentionally not observable through the public API.
