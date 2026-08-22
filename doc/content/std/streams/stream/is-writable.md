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
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
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
