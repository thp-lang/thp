---
kind: class
id: std.streams.Streams
title: Streams
summary: Opens dynamic stream URIs through a PHP-compatible mode bridge.
name: Streams
module: streams
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: partial
notice: Streams::open() implements php://memory, php://temp[/maxmemory:N], and
  read-only thp:/input. Local paths, file://, and complete mode support remain proposed.
version: "0.1"
---

`Streams` is a final compatibility factory for names computed at runtime. The
executable subset recognizes `php://memory`, `php://temp`,
`php://temp/maxmemory:N`, and the request's shared read-only `thp:/input`
stream. Local paths and `file://` remain proposed here.

The proposed general signature returns `Stream`. The executable compiler
currently gives supported literal URIs a narrower native stream type. Use
`instanceof` to narrow capability interfaces; the `Stream::isReadable()`,
`isWritable()`, and `isSeekable()` methods remain proposed.
