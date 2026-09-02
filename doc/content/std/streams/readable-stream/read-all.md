---
kind: method
id: std.streams.ReadableStream::readAll
title: ReadableStream::readAll
summary: Reads from the cursor through end of stream.
name: readAll
order: 20
typeParameters: []
parameters:
  - name: limit
    type: ?int
    description: Optional maximum remaining byte count.
    default: "null"
returns:
  type: string
  description: All remaining bytes through end of stream.
errors:
  - type: ValueError
    description: limit is negative.
  - type: IoException
    description: The remaining data exceeds limit; no data is consumed and the cursor is unchanged.
  - type: ClosedStreamException
    description: The shared stream handle has already been closed.
related:
  - std.streams.ReadableStream
status: experimental
availability: implemented
notice: The compiler and VM implement this operation, including non-consuming limit failure.
version: "0.1"
owner: std.streams.ReadableStream
visibility: public
modifiers: []
---

Reads from the cursor through end of stream.

## Behavior

Reads from the cursor through end of stream.

## Example

```thp
$contents = $stream->readAll(1048576);
```

## See also

- [`ReadableStream`](thp:std.streams.ReadableStream)
