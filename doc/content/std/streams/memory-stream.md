---
kind: class
id: std.streams.MemoryStream
title: MemoryStream
summary: Stores a readable, writable, seekable byte stream in memory.
name: MemoryStream
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

This is a final native stream class created through `MemoryStream::open()`.
It stores bytes in a geometrically growing memory buffer and starts with its cursor
at zero even when initial contents are supplied.

```thp
$stream = MemoryStream::open("\x00\xffTHP");
$stream->seek(2);
$stream->writeAll("typed");
$stream->seek(0);
$bytes = $stream->readAll();
```
