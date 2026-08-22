---
kind: method
id: std.spl.InfiniteIterator::getInnerIterator
title: InfiniteIterator::getInnerIterator
summary: Returns the iterator used for the current cycle.
name: getInnerIterator
order: 2
typeParameters: []
parameters: []
returns:
  type: ?Iterator<K, V>
  description: Returns the iterator used for the current cycle.
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
owner: std.spl.InfiniteIterator
visibility: public
modifiers: []
---

[`InfiniteIterator`](thp:std.spl.InfiniteIterator)`::getInnerIterator()` returns the iterator used for the current cycle.

## Behavior

Returns the iterator used for the current cycle.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getInnerIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`InfiniteIterator`](thp:std.spl.InfiniteIterator)
