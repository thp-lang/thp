---
kind: method
id: std.streams.Closeable::close
title: Closeable::close
summary: Closes the shared handle exactly once.
name: close
order: 10
typeParameters: []
parameters: []
returns:
  type: void
  description: This method does not return a value.
errors:
  - type: IoException
    description: The underlying input/output operation fails.
related:
  - std.streams.Closeable
status: experimental
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
owner: std.streams.Closeable
visibility: public
modifiers: []
---

Closes the shared handle exactly once.

## Behavior

The first call releases the native payload immediately. Later calls through any alias have no effect.

## Example

```thp
$stream = MemoryStream::open("data");
$stream->close();
$stream->close();
```

## See also

- [`Closeable`](thp:std.streams.Closeable)
