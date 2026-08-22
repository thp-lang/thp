---
kind: method
id: std.streams.OpenStreamException::getTarget
title: OpenStreamException::getTarget
summary: Returns the requested path or URI.
name: getTarget
order: 20
typeParameters: []
parameters: []
returns:
  type: string
  description: The target supplied during construction.
errors: []
related:
  - std.streams.OpenStreamException
status: experimental
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
owner: std.streams.OpenStreamException
visibility: public
modifiers: []
---

Returns the requested path or URI.

## Behavior

Returns the requested path or URI.

## Example

```thp
$result = $instance->getTarget();
```

## See also

- [`OpenStreamException`](thp:std.streams.OpenStreamException)
