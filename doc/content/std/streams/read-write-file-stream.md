---
kind: class
id: std.streams.ReadWriteFileStream
title: ReadWriteFileStream
summary: Represents a readable, writable, and seekable file handle.
name: ReadWriteFileStream
module: streams
typeParameters: []
interfaces:
  - id: std.streams.ReadableStream
  - id: std.streams.WritableStream
  - id: std.streams.SeekableStream
constants: []
properties: []
status: experimental
availability: implemented
notice: Files::openReadWrite() returns a readable, writable, seekable, and closeable file handle.
version: "0.1"
---

This final native handle is returned by
[`Files::openReadWrite()`](thp:std.streams.Files::openReadWrite) when both
capabilities are requested explicitly.
