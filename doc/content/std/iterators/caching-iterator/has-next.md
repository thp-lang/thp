---
kind: method
id: std.spl.CachingIterator::hasNext
title: CachingIterator::hasNext
summary: Reports whether lookahead found another entry.
name: hasNext
order: 3
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether lookahead found another entry.
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

[`CachingIterator`](thp:std.spl.CachingIterator)`::hasNext()` reports whether lookahead found another entry.

## Behavior

Reports whether lookahead found another entry.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->hasNext();
```

The call uses the signature and defaults documented above.

## See also

- [`CachingIterator`](thp:std.spl.CachingIterator)
