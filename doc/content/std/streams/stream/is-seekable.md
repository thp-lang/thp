---
kind: method
id: std.streams.Stream::isSeekable
title: Stream::isSeekable
summary: Reports whether the stream supports seeking.
name: isSeekable
order: 30
typeParameters: []
parameters: []
returns:
  type: bool
  description: True when the stream exposes seek operations.
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

Reports whether the stream supports seeking.

## Behavior

Reports whether the stream supports seeking.

## Example

```thp
$result = $instance->isSeekable();
```

## See also

- [`Stream`](thp:std.streams.Stream)
