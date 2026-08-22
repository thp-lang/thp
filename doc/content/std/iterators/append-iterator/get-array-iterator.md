---
kind: method
id: std.spl.AppendIterator::getArrayIterator
title: AppendIterator::getArrayIterator
summary: Returns a keyed iterator over the appended iterators.
name: getArrayIterator
order: 4
typeParameters: []
parameters: []
returns:
  type: ArrayIterator<int, Iterator<K, V>>
  description: Returns a keyed iterator over the appended iterators.
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
owner: std.spl.AppendIterator
visibility: public
modifiers: []
---

[`AppendIterator`](thp:std.spl.AppendIterator)`::getArrayIterator()` returns a keyed iterator over the appended iterators.

## Behavior

Returns a keyed iterator over the appended iterators.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getArrayIterator();
```

The call uses the signature and defaults documented above.

## See also

- [`AppendIterator`](thp:std.spl.AppendIterator)
