---
kind: method
id: std.baseTypes.Throwable::getFile
title: Throwable::getFile
summary: Returns the source file where it was created.
name: getFile
order: 3
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the source file where it was created.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
notice: This method executes in the reference VM.
version: "0.1"
owner: std.baseTypes.Throwable
visibility: public
modifiers: []
---

[`Throwable`](thp:std.baseTypes.Throwable)`::getFile()` returns the source file where it was created.

## Behavior

Returns the source file where it was created.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getFile();
```

The call uses the signature and defaults documented above.

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)
