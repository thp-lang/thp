---
kind: method
id: std.streams.Files::openWrite
title: Files::openWrite
summary: Opens a local file for writing.
name: openWrite
order: 20
typeParameters: []
parameters:
  - name: path
    type: string
    description: Local filesystem path to open.
  - name: mode
    type: WriteMode
    description: Creation and positioning behavior.
    default: WriteMode::Truncate
returns:
  type: WritableFileStream
  description: A writable and seekable file handle.
errors:
  - type: OpenStreamException
    description: The path cannot be opened under the selected mode.
related:
  - std.streams.Files
status: experimental
availability: proposed
notice: Writable-file opening is proposed and is not implemented.
version: "0.1"
owner: std.streams.Files
visibility: public
modifiers:
  - static
---

Opens a local file for writing.

## Behavior

Opens a local file for writing.

## Example

```thp
$stream = Files::openWrite("./report.txt", WriteMode::Truncate);
```

## See also

- [`Files`](thp:std.streams.Files)
