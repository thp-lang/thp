---
kind: method
id: std.spl.FilterIterator::getInnerIterator
title: FilterIterator::getInnerIterator
summary: Returns the wrapped iterator.
name: getInnerIterator
order: 3
typeParameters: []
parameters: []
returns:
  type: ?Iterator<K, V>
  description: Returns the wrapped iterator.
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
owner: std.spl.FilterIterator
visibility: public
modifiers: []
---

[`FilterIterator`](thp:std.spl.FilterIterator)`::getInnerIterator()` returns the wrapped iterator.

## Behavior

Returns the wrapped iterator.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getInnerIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`FilterIterator`](thp:std.spl.FilterIterator)
