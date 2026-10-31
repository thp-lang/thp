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
availability: implemented
notice: Streams::open() supports local paths, local file:// URIs, php://memory,
  php://temp[/maxmemory:N], and read-only thp:/input with mode-specific capabilities.
version: "0.1"
---

`Streams` is a final compatibility factory for names computed at runtime. The
executable subset recognizes local paths, local `file://` URIs,
`php://memory`, `php://temp`, `php://temp/maxmemory:N`, and the request's shared
read-only `thp:/input` stream. File URIs accept an empty or `localhost`
authority and percent-encoded UTF-8 paths.

The general signature returns `Stream`. The executable compiler gives the
request-input URI a narrower readable type.
Use `instanceof` to narrow capability interfaces; `Stream::isReadable()`,
`isWritable()`, and `isSeekable()` report the current runtime capabilities.
