---
kind: method
id: std.spl.ArrayIterator::count
title: ArrayIterator::count
summary: Returns the number of represented values.
name: count
order: 8
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
owner: std.spl.ArrayIterator
visibility: public
modifiers: []
---

[`ArrayIterator`](thp:std.spl.ArrayIterator)`::count()` returns the number of represented values.

## Behavior

Returns the number of represented values.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->count();
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayIterator`](thp:std.spl.ArrayIterator)
