---
kind: method
id: std.spl.CachingIterator::__toString
title: CachingIterator::__toString
summary: Formats the current entry according to flags.
name: __toString
order: 4
typeParameters: []
parameters: []
returns:
  type: string
  description: Formats the current entry according to flags.
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
owner: std.spl.CachingIterator
visibility: public
modifiers: []
---

[`CachingIterator`](thp:std.spl.CachingIterator)`::__toString()` formats the current entry according to flags.

## Behavior

Formats the current entry according to flags.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->__toString();
```

The call uses the signature and defaults documented above.

## See also

- [`CachingIterator`](thp:std.spl.CachingIterator)
