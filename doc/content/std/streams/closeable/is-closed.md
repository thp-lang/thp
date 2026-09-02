---
kind: method
id: std.streams.Closeable::isClosed
title: Closeable::isClosed
summary: Reports whether the shared handle is closed.
name: isClosed
order: 20
typeParameters: []
parameters: []
returns:
  type: bool
  description: True after the payload has been closed; otherwise false.
errors: []
related:
  - std.streams.Closeable
status: experimental
availability: implemented
notice: The compiler and VM implement shared close-state inspection.
version: "0.1"
owner: std.streams.Closeable
visibility: public
modifiers: []
---

Reports whether the shared handle is closed.

## Behavior

Every alias reports the same close state.

## Example

```thp
$result = $instance->isClosed();
```

## See also

- [`Closeable`](thp:std.streams.Closeable)
