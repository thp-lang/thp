---
kind: method
id: std.spl.GlobIterator::count
title: GlobIterator::count
summary: Returns the number of represented values.
name: count
order: 2
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the number of represented values.
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
owner: std.spl.GlobIterator
visibility: public
modifiers: []
---

[`GlobIterator`](thp:std.spl.GlobIterator)`::count()` returns the number of represented values.

## Behavior

Returns the number of represented values.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->count();
```

The call uses the signature and defaults documented above.

## See also

- [`GlobIterator`](thp:std.spl.GlobIterator)
