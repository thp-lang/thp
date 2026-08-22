---
kind: method
id: std.spl.ArrayObject::getFlags
title: ArrayObject::getFlags
summary: Returns configured behavior flags.
name: getFlags
order: 9
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns configured behavior flags.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.ArrayObject
visibility: public
modifiers: []
---

[`ArrayObject`](thp:std.spl.ArrayObject)`::getFlags()` returns configured behavior flags.

## Behavior

Returns configured behavior flags.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getFlags();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayObject`](thp:std.spl.ArrayObject)
