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
availability: implemented
notice: Files::openWrite() returns a writable, seekable, and closeable file handle.
version: "0.1"
---

This final native handle is returned by
[`Files::openWrite()`](thp:std.streams.Files::openWrite). Its static type omits
read operations.
