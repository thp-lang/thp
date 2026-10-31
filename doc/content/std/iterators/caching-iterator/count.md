---
kind: method
id: std.spl.CachingIterator::count
title: CachingIterator::count
summary: Returns the number of entries visited in the current traversal.
name: count
order: 12
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the number of entries visited in the current traversal.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: implemented
notice: This method is executable in the reference VM.
version: "0.1"
owner: std.spl.CachingIterator
visibility: public
modifiers: []
---

[`CachingIterator`](thp:std.spl.CachingIterator)`::count()` returns the number of entries visited in the current traversal.

## Behavior

Returns the number of entries visited in the current traversal.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->count();
```

The call uses the signature and defaults documented above.

## See also

- [`CachingIterator`](thp:std.spl.CachingIterator)
