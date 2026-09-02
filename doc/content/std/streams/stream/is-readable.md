---
kind: method
id: std.streams.Stream::isReadable
title: Stream::isReadable
summary: Reports whether the stream supports reading.
name: isReadable
order: 10
typeParameters: []
parameters: []
returns:
  type: bool
  description: True when the stream exposes readable operations.
errors: []
related:
  - std.streams.Stream
status: experimental
availability: proposed
notice: This capability-inspection method is proposed and is not implemented; use instanceof.
version: "0.1"
owner: std.streams.Stream
visibility: public
modifiers: []
---

Reports whether the stream supports reading.

## Behavior

Reports whether the stream supports reading.

## Example

```thp
$result = $instance->isReadable();
```

## See also

- [`Stream`](thp:std.streams.Stream)
