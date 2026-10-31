---
kind: method
id: std.streams.OpenStreamException::__construct
title: OpenStreamException::__construct
summary: Creates an opening failure with its requested target and platform code.
name: __construct
order: 10
typeParameters: []
parameters:
  - name: message
    type: string
    description: Human-readable diagnostic message.
  - name: target
    type: string
    description: Requested path or URI.
  - name: systemCode
    type: int
    description: Platform error code, or zero when unavailable.
    default: "0"
  - name: previous
    type: ?Throwable
    description: Preceding failure, when available.
    default: "null"
returns:
  type: void
  description: This method does not return a value.
errors: []
related:
  - std.streams.OpenStreamException
status: experimental
availability: partial
notice: The VM constructs this state for runtime failures. The documented public
  four-argument constructor is not implemented.
version: "0.1"
owner: std.streams.OpenStreamException
visibility: public
modifiers: []
---

Creates an opening failure with its requested target and platform code.

## Behavior

Creates an opening failure with its requested target and platform code.

## Example

```thp
$error = new OpenStreamException(
    "cannot open stream",
    "./missing",
    2,
);
```

## See also

- [`OpenStreamException`](thp:std.streams.OpenStreamException)
