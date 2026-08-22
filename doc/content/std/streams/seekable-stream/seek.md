---
kind: method
id: std.streams.SeekableStream::seek
title: SeekableStream::seek
summary: Moves the cursor and returns its absolute position.
name: seek
order: 10
typeParameters: []
parameters:
  - name: offset
    type: int
    description: Signed byte offset from the selected origin.
  - name: from
    type: SeekFrom
    description: Origin used to interpret offset.
    default: SeekFrom::Start
returns:
  type: int
  description: The new absolute byte position.
errors:
  - type: ValueError
    description: The resulting absolute position would be negative.
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

Moves the cursor and returns its absolute position.

## Behavior

Moves the cursor and returns its absolute position.

## Example

```thp
$position = $stream->seek(0, SeekFrom::End);
```

## See also

- [`SeekableStream`](thp:std.streams.SeekableStream)
