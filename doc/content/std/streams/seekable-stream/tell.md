---
kind: method
id: std.streams.SeekableStream::tell
title: SeekableStream::tell
summary: Returns the absolute cursor position.
name: tell
order: 20
typeParameters: []
parameters: []
returns:
  type: int
  description: The current zero-based byte position.
errors:
  - type: ClosedStreamException
    description: The shared stream handle has already been closed.
  - type: IoException
    description: The underlying input/output operation fails.
related:
  - std.streams.SeekableStream
status: experimental
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
owner: std.streams.SeekableStream
visibility: public
modifiers: []
---

Returns the absolute cursor position.

## Behavior

Returns the absolute cursor position.

## Example

```thp
$result = $instance->tell();
```

## See also

- [`SeekableStream`](thp:std.streams.SeekableStream)
