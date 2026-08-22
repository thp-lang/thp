---
kind: class
id: std.streams.WritableFileStream
title: WritableFileStream
summary: Represents a statically writable and seekable file handle.
name: WritableFileStream
module: streams
typeParameters: []
interfaces:
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

This final native handle is returned by
[`Files::openWrite()`](thp:std.streams.Files::openWrite). Its static type omits
read operations.
