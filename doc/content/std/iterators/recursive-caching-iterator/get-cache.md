---
kind: method
id: std.spl.RecursiveCachingIterator::getCache
title: RecursiveCachingIterator::getCache
summary: Returns entries retained so far.
name: getCache
order: 3
typeParameters: []
parameters: []
returns:
  type: vector<RecursiveEntry<T>>
  description: Returns entries retained so far.
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
owner: std.spl.RecursiveCachingIterator
visibility: public
modifiers: []
---

[`RecursiveCachingIterator`](thp:std.spl.RecursiveCachingIterator)`::getCache()` returns entries retained so far.

## Behavior

Returns entries retained so far.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getCache();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveCachingIterator`](thp:std.spl.RecursiveCachingIterator)
