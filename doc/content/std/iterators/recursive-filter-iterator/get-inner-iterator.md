---
kind: method
id: std.spl.RecursiveFilterIterator::getInnerIterator
title: RecursiveFilterIterator::getInnerIterator
summary: Returns the wrapped recursive iterator.
name: getInnerIterator
order: 3
typeParameters: []
parameters: []
returns:
  type: ?Iterator<K, RecursiveEntry<T>>
  description: Returns the wrapped recursive iterator.
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
owner: std.spl.RecursiveFilterIterator
visibility: public
modifiers: []
---

[`RecursiveFilterIterator`](thp:std.spl.RecursiveFilterIterator)`::getInnerIterator()` returns the wrapped recursive iterator.

## Behavior

Returns the wrapped recursive iterator.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getInnerIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveFilterIterator`](thp:std.spl.RecursiveFilterIterator)
