---
kind: method
id: std.streams.Streams::open
title: Streams::open
summary: Opens a dynamic path or stream URI.
name: open
order: 10
typeParameters: []
parameters:
  - name: uri
    type: string
    description: Path or supported stream URI.
  - name: mode
    type: OpenMode
    description: Requested dynamic capabilities.
returns:
  type: Stream
  description: A stream whose runtime interfaces reflect the requested capabilities.
errors:
  - type: InvalidStreamUriException
    description: The scheme or wrapper option is unknown or malformed.
  - type: UnsupportedStreamOperationException
    description: The target cannot provide a requested capability.
  - type: OpenStreamException
    description: The recognized target cannot be opened.
related:
  - std.streams.Streams
status: experimental
availability: partial
notice:
  Implements php://memory, php://temp[/maxmemory:N], and read-only thp:/input.
  Local paths, file://, capability inspection, and complete OpenMode behavior
  remain proposed.
version: "0.1"
owner: std.streams.Streams
visibility: public
modifiers:
  - static
---

Opens a dynamic path or stream URI.

## Behavior

`php://memory` and `php://temp/maxmemory:N` create independent stream cells.
`thp:/input` accepts only `OpenMode::Read` and aliases the single request body
cell and cursor. It is readable and closeable but not writable or seekable.

An unsupported mode throws `UnsupportedStreamOperationException`. An unknown
or malformed URI throws `InvalidStreamUriException`.

## Example

```thp
$input = Streams::open("thp:/input", OpenMode::Read);
echo $input->readAll();
```

## See also

- [`Streams`](thp:std.streams.Streams)
- [`ReadableStream`](thp:std.streams.ReadableStream)
