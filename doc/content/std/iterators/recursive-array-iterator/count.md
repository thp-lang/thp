---
kind: method
id: std.spl.RecursiveArrayIterator::count
title: RecursiveArrayIterator::count
summary: Returns the number of entries in the copied top-level collection.
name: count
order: 6
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the number of entries in the copied top-level collection.
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
owner: std.spl.RecursiveArrayIterator
visibility: public
modifiers: []
---

[`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)`::count()` returns the number of entries in the copied top-level collection.

## Behavior

Returns the number of entries in the copied top-level collection.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->count();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveArrayIterator`](thp:std.spl.RecursiveArrayIterator)
