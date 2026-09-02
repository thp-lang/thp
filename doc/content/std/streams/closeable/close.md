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
availability: implemented
notice: The compiler and VM implement shared, idempotent stream closing.
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
