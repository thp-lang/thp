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
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  partially implemented experimental contract and may change as runtime integration proceeds.
version: "0.1"
---

`Streams` is a final compatibility factory for names computed at runtime.
The first proposal recognizes local paths, `file://`, `php://memory`, and
`php://temp/maxmemory:N`. The executable runtime also provides the request's
shared, read-only `thp:/input` stream.

Because the URI and mode are dynamic, `open()` returns `Stream`. Use
`instanceof` to narrow the result to its supported capability interfaces.
