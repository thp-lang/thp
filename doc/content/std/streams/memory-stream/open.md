---
kind: method
id: std.streams.MemoryStream::open
title: MemoryStream::open
summary: Creates an in-memory stream with its cursor at zero.
name: open
order: 10
typeParameters: []
parameters:
  - name: initial
    type: string
    description: Initial byte contents.
    default: '""'
returns:
  type: MemoryStream
  description: A new readable, writable, seekable memory stream.
errors: []
related:
  - std.streams.MemoryStream
status: experimental
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
owner: std.streams.MemoryStream
visibility: public
modifiers:
  - static
---

Creates an in-memory stream with its cursor at zero.

## Behavior

Creates an in-memory stream with its cursor at zero.

## Example

```thp
$stream = MemoryStream::open("\x00\xffTHP");
```

## See also

- [`MemoryStream`](thp:std.streams.MemoryStream)
