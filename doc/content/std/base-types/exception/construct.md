---
kind: method
id: std.baseTypes.Exception::__construct
title: Exception::__construct
summary: Initializes an exception's message, code, and previous throwable.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: message
    type: string
    description: Exception message.
    default: '""'
  - name: code
    type: int
    description: Exception code.
    default: "0"
  - name: previous
    type: ?Throwable
    description: Previously thrown object.
    default: "null"
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description: Argument types are checked statically.
related: []
status: experimental
availability: partial
notice:
  The compiler and reference VM implement this constructor. Source origin and
  trace capture remain proposed.
version: "0.1"
owner: std.baseTypes.Exception
visibility: public
modifiers: []
---

[`Exception`](thp:std.baseTypes.Exception)`::__construct()` initializes the
common state shared by native exceptions and user-defined descendants.

## Behavior

The message defaults to an empty string, the code defaults to zero, and the
previous throwable defaults to `null`. A child constructor must call
`parent::__construct()` explicitly when it wants this initialization.

## Example

```thp
$failure = new Exception("Invalid input", 7);
```

The call uses the signature and defaults documented above.

## See also

- [`Exception`](thp:std.baseTypes.Exception)
