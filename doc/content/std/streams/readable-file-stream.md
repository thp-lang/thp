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
notice: Files::openRead(), reading, tell(), and closing execute. The documented
  SeekableStream relationship and seek() operation are not implemented for files.
version: "0.1"
---

This final native handle is returned by
[`Files::openRead()`](thp:std.streams.Files::openRead). Its static type omits
write operations, so unsupported writes are rejected during compilation.
