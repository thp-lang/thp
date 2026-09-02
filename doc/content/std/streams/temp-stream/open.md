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
availability: implemented
notice: The compiler and VM implement this factory, threshold validation, and one-time spill.
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
