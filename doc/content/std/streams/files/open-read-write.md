---
kind: method
id: std.streams.Files::openReadWrite
title: Files::openReadWrite
summary: Opens a local file for reading and writing.
name: openReadWrite
order: 30
typeParameters: []
parameters:
  - name: path
    type: string
    description: Local filesystem path to open.
  - name: mode
    type: WriteMode
    description: Creation and positioning behavior.
    default: WriteMode::OpenExisting
returns:
  type: ReadWriteFileStream
  description: A readable, writable, and seekable file handle.
errors:
  - type: OpenStreamException
    description: The path cannot be opened under the selected mode.
related:
  - std.streams.Files
status: experimental
availability: proposed
notice: Read-write file opening is proposed and is not implemented.
version: "0.1"
owner: std.streams.Files
visibility: public
modifiers:
  - static
---

Opens a local file for reading and writing.

## Behavior

Opens a local file for reading and writing.

## Example

```thp
$stream = Files::openReadWrite("./state.bin");
```

## See also

- [`Files`](thp:std.streams.Files)
