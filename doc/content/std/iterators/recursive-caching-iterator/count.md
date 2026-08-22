---
kind: method
id: std.spl.RecursiveCachingIterator::count
title: RecursiveCachingIterator::count
summary: Returns retained entry count.
name: count
order: 4
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns retained entry count.
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

[`RecursiveCachingIterator`](thp:std.spl.RecursiveCachingIterator)`::count()` returns retained entry count.

## Behavior

Returns retained entry count.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->count();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveCachingIterator`](thp:std.spl.RecursiveCachingIterator)
