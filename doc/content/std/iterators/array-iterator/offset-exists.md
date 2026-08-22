---
kind: method
id: std.spl.ArrayIterator::offsetExists
title: ArrayIterator::offsetExists
summary: Reports whether an offset exists.
name: offsetExists
order: 2
typeParameters: []
parameters:
  - name: key
    type: K
    description: Key addressed by the operation.
returns:
  type: bool
  description: Reports whether an offset exists.
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

[`ArrayIterator`](thp:std.spl.ArrayIterator)`::offsetExists()` reports whether an offset exists.

## Behavior

Reports whether an offset exists.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->offsetExists($key);
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayIterator`](thp:std.spl.ArrayIterator)
