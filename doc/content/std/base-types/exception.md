---
kind: class
id: std.baseTypes.Exception
title: Exception
summary: Base class for user-defined exceptional conditions.
name: Exception
module: base-types
typeParameters: []
interfaces:
  - id: std.baseTypes.Throwable
constants: []
properties: []
status: experimental
availability: partial
notice: >-
  The compiler and reference VM implement construction, nominal subtype
  catching, message, code, previous, and suppressed-failure state. Source
  origin, trace inspection, cloning rules, and string conversion remain
  proposed.
version: "0.1"
---

`Exception` is the base class for exceptions defined and thrown by application
code.

## Construction

| Method                                                      | Description                                                    |
| ----------------------------------------------------------- | -------------------------------------------------------------- |
| [`__construct()`](thp:std.baseTypes.Exception::__construct) | Initializes message, code, and an optional previous throwable. |

## Behavior

Application exception classes extend `Exception`. A `catch (Exception ...)`
clause accepts this class and its subclasses, but does not catch the separate
`Error` hierarchy. Catch `Throwable` when both roots are intended. The current
executable slice does not expose exception state as ordinary object properties.

## Example

```thp
class ConfigurationException extends Exception
{
}

function requireSetting(?string $value): string {
    if ($value === null) {
        throw new ConfigurationException("Missing setting");
    }

    return $value;
}
```

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)
- `Error`
- `ErrorException`
