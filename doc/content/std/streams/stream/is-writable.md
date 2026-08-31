---
kind: method
id: std.streams.Stream::isWritable
title: Stream::isWritable
summary: Reports whether the stream supports writing.
name: isWritable
order: 20
typeParameters: []
parameters: []
returns:
  type: bool
  description: True when the stream exposes writable operations.
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

Reports whether the stream supports writing.

## Behavior

Reports whether the stream supports writing.

## Example

```thp
$result = $instance->isWritable();
```

## See also

- [`Stream`](thp:std.streams.Stream)
