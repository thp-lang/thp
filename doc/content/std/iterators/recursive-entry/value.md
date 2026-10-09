---
kind: method
id: std.spl.RecursiveEntry::value
title: RecursiveEntry::value
summary: Returns the value represented by this entry.
name: value
order: 1
typeParameters: []
parameters: []
returns:
  type: T
  description: Returns the value represented by this entry.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
notice: This method is executable in the reference VM.
version: "0.1"
owner: std.spl.RecursiveEntry
visibility: public
modifiers: []
---

[`RecursiveEntry`](thp:std.spl.RecursiveEntry)`::value()` returns the value represented by this entry.

## Behavior

Returns the value represented by this entry.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->value();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveEntry`](thp:std.spl.RecursiveEntry)
