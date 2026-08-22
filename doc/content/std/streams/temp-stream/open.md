---
kind: method
id: std.streams.TempStream::open
title: TempStream::open
summary: Creates a stream that may spill to a temporary file.
name: open
order: 10
typeParameters: []
parameters:
  - name: maxMemoryBytes
    type: int
    description: Largest in-memory size before spilling to disk.
    default: "2097152"
returns:
  type: TempStream
  description: A new readable, writable, seekable temporary stream.
errors:
  - type: ValueError
    description: maxMemoryBytes is negative.
  - type: IoException
    description: The underlying input/output operation fails.
related:
  - std.streams.TempStream
status: experimental
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
owner: std.streams.TempStream
visibility: public
modifiers:
  - static
---

Creates a stream that may spill to a temporary file.

## Behavior

A zero threshold uses a temporary file on the first write. At most one memory-to-file spill occurs.

## Example

```thp
$stream = TempStream::open(4096);
```

## See also

- [`TempStream`](thp:std.streams.TempStream)
