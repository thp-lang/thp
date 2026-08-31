---
kind: class
id: std.streams.Files
title: Files
summary: Opens local files with statically known capabilities.
name: Files
module: streams
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: partial
notice:
  Files::openRead() is implemented. Writing factories, WriteMode, and writable
  or read-write file handles remain proposed.
version: "0.1"
---

`Files` is a final factory class. Separate methods return readable,
writable, or read-write handle types, avoiding mode-string casts and making
unsupported operations compile-time errors.
