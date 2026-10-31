---
kind: method
id: std.streams.OpenStreamException::getSystemCode
title: OpenStreamException::getSystemCode
summary: Returns the platform opening error code.
name: getSystemCode
order: 30
typeParameters: []
parameters: []
returns:
  type: int
  description: The platform error code, or zero when unavailable.
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

Returns the platform opening error code.

## Behavior

Returns the platform opening error code.

## Example

```thp
$result = $instance->getSystemCode();
```

## See also

- [`OpenStreamException`](thp:std.streams.OpenStreamException)
