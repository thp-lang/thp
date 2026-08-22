---
kind: method
id: std.baseTypes.Throwable::getPrevious
title: Throwable::getPrevious
summary: Returns the preceding throwable, when available.
name: getPrevious
order: 7
typeParameters: []
parameters: []
returns:
  type: ?Throwable
  description: Returns the preceding throwable, when available.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: partial
notice:
  The compiler and reference VM implement this member for Exception, Error, and
  their descendants.
version: "0.1"
owner: std.baseTypes.Throwable
visibility: public
modifiers: []
---

[`Throwable`](thp:std.baseTypes.Throwable)`::getPrevious()` returns the preceding throwable, when available.

## Behavior

Returns the preceding throwable, when available.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$root = new Exception("root");
$result = (new Exception("replacement", 0, $root))->getPrevious();
```

The call uses the signature and defaults documented above.

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)
