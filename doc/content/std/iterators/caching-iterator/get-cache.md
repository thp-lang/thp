---
kind: method
id: std.spl.CachingIterator::getCache
title: CachingIterator::getCache
summary: Returns entries retained by full-cache mode.
name: getCache
order: 11
typeParameters: []
parameters: []
returns:
  type: map<K, V>
  description: Returns entries retained by full-cache mode.
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

[`CachingIterator`](thp:std.spl.CachingIterator)`::getCache()` returns entries retained by full-cache mode.

## Behavior

Returns entries retained by full-cache mode.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getCache();
```

The call uses the signature and defaults documented above.

## See also

- [`CachingIterator`](thp:std.spl.CachingIterator)
