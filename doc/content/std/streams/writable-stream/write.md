---
kind: method
id: std.streams.WritableStream::write
title: WritableStream::write
summary: Writes a prefix of a byte string.
name: write
order: 10
typeParameters: []
parameters:
  - name: data
    type: string
    description: Bytes offered to the stream.
returns:
  type: int
  description: The number of bytes accepted, which may be less than the input.
errors:
  - type: ClosedStreamException
    description: The shared stream handle has already been closed.
  - type: IoException
    description: The underlying input/output operation fails.
related:
  - std.streams.WritableStream
status: experimental
availability: implemented
notice: Native streams currently accept the complete byte string or throw an I/O exception.
version: "0.1"
owner: std.streams.WritableStream
visibility: public
modifiers: []
---

Writes a prefix of a byte string.

## Behavior

Writes a prefix of a byte string.

## Example

```thp
$written = $stream->write("payload");
```

## See also

- [`WritableStream`](thp:std.streams.WritableStream)
