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
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
---

This is a final native stream class created through `TempStream::open()`.
It behaves like `MemoryStream` until a write would exceed the configured memory
threshold, then copies its contents once to an anonymous temporary file while
preserving the cursor.

The selected backend is intentionally not observable through the public API.
