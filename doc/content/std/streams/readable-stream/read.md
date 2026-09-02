---
kind: method
id: std.streams.ReadableStream::read
title: ReadableStream::read
summary: Reads up to a requested number of bytes.
name: read
order: 10
typeParameters: []
parameters:
  - name: length
    type: int
    description: Maximum number of bytes to read.
returns:
  type: string
  description: Up to length bytes, an empty string for a zero request or at end of stream.
errors:
  - type: ValueError
    description: length is negative.
  - type: ClosedStreamException
    description: The shared stream handle has already been closed.
  - type: IoException
    description: The underlying input/output operation fails.
related:
  - std.streams.ReadableStream
status: experimental
availability: implemented
notice: The compiler and VM implement this operation and its documented cursor behavior.
version: "0.1"
owner: std.streams.ReadableStream
visibility: public
modifiers: []
---

Reads up to a requested number of bytes.

## Behavior

The cursor advances by the returned byte count. read(0) does not change the cursor.

## Example

```thp
$chunk = $stream->read(4096);
```

## See also

- [`ReadableStream`](thp:std.streams.ReadableStream)
