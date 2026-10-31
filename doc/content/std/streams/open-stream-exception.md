---
kind: class
id: std.streams.OpenStreamException
title: OpenStreamException
summary: Reports failure to open a path or stream URI.
name: OpenStreamException
module: streams
typeParameters: []
parent:
  id: std.streams.IoException
interfaces: []
constants: []
properties: []
status: experimental
availability: implemented
notice: The VM produces this exception with target and system-code state when opening fails.
version: "0.1"
---

`OpenStreamException` retains the requested path or URI and a platform
error code when one is available. A zero system code means no platform code was
reported.
