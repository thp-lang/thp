---
kind: method
id: std.spl.CachingIterator::setFlags
title: CachingIterator::setFlags
summary: Replaces caching and formatting flags.
name: setFlags
order: 6
typeParameters: []
parameters:
  - name: flags
    type: int
    description: Bit mask selecting the documented options.
returns:
  type: void
  description: This method does not return a value.
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

[`CachingIterator`](thp:std.spl.CachingIterator)`::setFlags()` replaces caching and formatting flags.

## Behavior

Replaces caching and formatting flags.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setFlags($flags);
```

The call uses the signature and defaults documented above.

## See also

- [`CachingIterator`](thp:std.spl.CachingIterator)
