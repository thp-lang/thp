---
kind: method
id: std.spl.MultipleIterator::containsIterator
title: MultipleIterator::containsIterator
summary: Reports whether an iterator is attached.
name: containsIterator
order: 5
typeParameters: []
parameters:
  - name: iterator
    type: Iterator<int, T>
    description: Iterator wrapped or consumed by this operation.
returns:
  type: bool
  description: Reports whether an iterator is attached.
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

[`MultipleIterator`](thp:std.spl.MultipleIterator)`::containsIterator()` reports whether an iterator is attached.

## Behavior

Reports whether an iterator is attached.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->containsIterator($iterator);
```

The call uses the signature and defaults documented above.

## See also

- [`MultipleIterator`](thp:std.spl.MultipleIterator)
