---
kind: method
id: std.streams.ReadableStream::eof
title: ReadableStream::eof
summary: Reports whether the cursor is at end of stream.
name: eof
order: 30
typeParameters: []
parameters: []
returns:
  type: bool
  description: True when no more bytes remain; otherwise false.
errors:
  - type: ClosedStreamException
    description: The shared stream handle has already been closed.
  - type: IoException
    description: The underlying input/output operation fails.
related:
  - std.streams.ReadableStream
status: experimental
availability: implemented
notice: The compiler and VM implement end-of-stream inspection.
version: "0.1"
owner: std.streams.ReadableStream
visibility: public
modifiers: []
---

Reports whether the cursor is at end of stream.

## Behavior

Reports whether the cursor is at end of stream.

## Example

```thp
$result = $instance->eof();
```

## See also

- [`ReadableStream`](thp:std.streams.ReadableStream)
