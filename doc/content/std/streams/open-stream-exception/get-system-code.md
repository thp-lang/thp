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

Returns the platform opening error code.

## Behavior

Returns the platform opening error code.

## Example

```thp
$result = $instance->getSystemCode();
```

## See also

- [`OpenStreamException`](thp:std.streams.OpenStreamException)
