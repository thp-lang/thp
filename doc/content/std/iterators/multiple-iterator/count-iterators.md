---
kind: method
id: std.spl.MultipleIterator::countIterators
title: MultipleIterator::countIterators
summary: Returns the number of attached iterators.
name: countIterators
order: 6
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the number of attached iterators.
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
owner: std.spl.MultipleIterator
visibility: public
modifiers: []
---

[`MultipleIterator`](thp:std.spl.MultipleIterator)`::countIterators()` returns the number of attached iterators.

## Behavior

Returns the number of attached iterators.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->countIterators();
```

The call uses the signature and defaults documented above.

## See also

- [`MultipleIterator`](thp:std.spl.MultipleIterator)
