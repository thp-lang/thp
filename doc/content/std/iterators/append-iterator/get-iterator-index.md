---
kind: method
id: std.spl.AppendIterator::getIteratorIndex
title: AppendIterator::getIteratorIndex
summary: Returns the active iterator index, or null before traversal.
name: getIteratorIndex
order: 3
typeParameters: []
parameters: []
returns:
  type: ?int
  description: Returns the active iterator index, or null before traversal.
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

[`AppendIterator`](thp:std.spl.AppendIterator)`::getIteratorIndex()` returns the active iterator index, or null before traversal.

## Behavior

Returns the active iterator index, or null before traversal.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getIteratorIndex();
```

The call uses the signature and defaults documented above.

## See also

- [`AppendIterator`](thp:std.spl.AppendIterator)
