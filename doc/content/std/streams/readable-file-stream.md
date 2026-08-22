---
kind: class
id: std.streams.ReadableFileStream
title: ReadableFileStream
summary: Represents a statically readable and seekable file handle.
name: ReadableFileStream
module: streams
typeParameters: []
interfaces:
  - id: std.streams.ReadableStream
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

This final native handle is returned by
[`Files::openRead()`](thp:std.streams.Files::openRead). Its static type omits
write operations, so unsupported writes are rejected during compilation.
