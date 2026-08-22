---
kind: method
id: std.spl.ArrayIterator::setFlags
title: ArrayIterator::setFlags
summary: Replaces configured behavior flags.
name: setFlags
order: 10
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
owner: std.spl.ArrayIterator
visibility: public
modifiers: []
---

[`ArrayIterator`](thp:std.spl.ArrayIterator)`::setFlags()` replaces configured behavior flags.

## Behavior

Replaces configured behavior flags.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setFlags($flags);
```

The call uses the signature and defaults documented above.

## See also

- [`ArrayIterator`](thp:std.spl.ArrayIterator)
