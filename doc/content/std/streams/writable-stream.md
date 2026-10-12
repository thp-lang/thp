---
kind: interface
id: std.streams.WritableStream
title: WritableStream
summary: Adds partial writes, complete writes, and buffered-output flushing.
name: WritableStream
module: streams
typeParameters: []
interfaces:
  - id: std.streams.Stream
constants: []
properties: []
status: experimental
availability: implemented
notice: Writing and flushing execute on memory, temporary, writable-file, and read-write-file streams.
version: "0.1"
---

`write()` may accept only a prefix and reports the accepted byte count.
`writeAll()` continues until every byte is accepted or an I/O failure occurs.
`flush()` makes buffered failures observable.
