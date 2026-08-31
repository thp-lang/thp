---
kind: method
id: std.streams.WritableStream::writeAll
title: WritableStream::writeAll
summary: Writes every byte in a string.
name: writeAll
order: 20
typeParameters: []
parameters:
  - name: data
    type: string
    description: Bytes written to the stream.
returns:
  type: void
  description: This method does not return a value.
errors:
  - type: ClosedStreamException
    description: The shared stream handle has already been closed.
  - type: IoException
    description: The underlying input/output operation fails.
related:
  - std.streams.WritableStream
status: experimental
availability: implemented
notice: The compiler and VM implement complete writes for memory and temporary streams.
version: "0.1"
owner: std.streams.WritableStream
visibility: public
modifiers: []
---

Writes every byte in a string.

## Behavior

The method repeats partial writes until all bytes are accepted or an I/O failure occurs.

## Example

```thp
$stream->writeAll("report\n");
```

## See also

- [`WritableStream`](thp:std.streams.WritableStream)
