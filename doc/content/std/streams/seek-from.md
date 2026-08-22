---
kind: enum
id: std.streams.SeekFrom
title: SeekFrom
summary: Selects the origin used by a seek operation.
name: SeekFrom
module: streams
typeParameters: []
interfaces: []
constants: []
properties: []
cases:
  - Start
  - Current
  - End
status: experimental
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
---

| Case      | Origin                         |
| --------- | ------------------------------ |
| `Start`   | The beginning of the stream.   |
| `Current` | The current cursor position.   |
| `End`     | The current end of the stream. |
