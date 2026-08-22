---
kind: method
id: std.streams.WritableStream::flush
title: WritableStream::flush
summary: Flushes buffered output and reports failures.
name: flush
order: 30
typeParameters: []
parameters: []
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
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
owner: std.streams.WritableStream
visibility: public
modifiers: []
---

Flushes buffered output and reports failures.

## Behavior

Flushes buffered output and reports failures.

## Example

```thp
$stream->flush();
```

## See also

- [`WritableStream`](thp:std.streams.WritableStream)
