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
availability: proposed
notice: Read-write file handles are proposed and are not implemented.
version: "0.1"
---

This final native handle is returned by
[`Files::openReadWrite()`](thp:std.streams.Files::openReadWrite) when both
capabilities are requested explicitly.
