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
availability: implemented
notice: The compiler and VM implement this accessor for runtime-produced opening failures.
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
