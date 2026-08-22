---
kind: method
id: std.spl.SplObjectStorage::getIterator
title: SplObjectStorage::getIterator
summary: Returns object-information entries in insertion order.
name: getIterator
order: 14
typeParameters: []
parameters: []
returns:
  type: Iterator<object, ?TInfo>
  description: Returns object-information entries in insertion order.
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
owner: std.spl.SplObjectStorage
visibility: public
modifiers: []
---

[`SplObjectStorage`](thp:std.spl.SplObjectStorage)`::getIterator()` returns object-information entries in insertion order.

## Behavior

Returns object-information entries in insertion order.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`SplObjectStorage`](thp:std.spl.SplObjectStorage)
