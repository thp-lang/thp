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
availability: proposed
notice: Relative and end-origin seeking are proposed and are not implemented.
version: "0.1"
---

| Case      | Origin                         |
| --------- | ------------------------------ |
| `Start`   | The beginning of the stream.   |
| `Current` | The current cursor position.   |
| `End`     | The current end of the stream. |
