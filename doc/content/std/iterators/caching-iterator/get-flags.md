---
kind: method
id: std.spl.CachingIterator::getFlags
title: CachingIterator::getFlags
summary: Returns caching and formatting flags.
name: getFlags
order: 5
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns caching and formatting flags.
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

[`CachingIterator`](thp:std.spl.CachingIterator)`::getFlags()` returns caching and formatting flags.

## Behavior

Returns caching and formatting flags.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getFlags();
```

The call uses the signature and defaults documented above.

## See also

- [`CachingIterator`](thp:std.spl.CachingIterator)
