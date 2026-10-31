---
kind: method
id: std.streams.Files::openRead
title: Files::openRead
summary: Opens an existing local file for reading.
name: openRead
order: 10
typeParameters: []
parameters:
  - name: path
    type: string
    description: Local filesystem path to open.
returns:
  type: ReadableFileStream
  description: A readable and seekable file handle.
errors:
  - type: OpenStreamException
    description: The path is missing or cannot be opened for reading.
related:
  - std.streams.Files
status: experimental
availability: implemented
notice: The compiler and VM implement read-only local-file opening and typed failures.
version: "0.1"
owner: std.streams.Files
visibility: public
modifiers:
  - static
---

Opens an existing local file for reading.

## Behavior

Opens an existing local file for reading.

## Example

```thp
$stream = Files::openRead("./input.bin");
```

## See also

- [`Files`](thp:std.streams.Files)
