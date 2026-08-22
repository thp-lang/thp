---
kind: method
id: std.spl.LimitIterator::getPosition
title: LimitIterator::getPosition
summary: Returns the number of values yielded by this window.
name: getPosition
order: 4
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the number of values yielded by this window.
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
owner: std.spl.LimitIterator
visibility: public
modifiers: []
---

[`LimitIterator`](thp:std.spl.LimitIterator)`::getPosition()` returns the number of values yielded by this window.

## Behavior

Returns the number of values yielded by this window.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getPosition();
```

The call uses the signature and defaults documented above.

## See also

- [`LimitIterator`](thp:std.spl.LimitIterator)
