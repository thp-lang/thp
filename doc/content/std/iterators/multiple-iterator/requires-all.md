---
kind: method
id: std.spl.MultipleIterator::requiresAll
title: MultipleIterator::requiresAll
summary: Reports the exhaustion policy.
name: requiresAll
order: 2
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports the exhaustion policy.
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

[`MultipleIterator`](thp:std.spl.MultipleIterator)`::requiresAll()` reports the exhaustion policy.

## Behavior

Reports the exhaustion policy.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->requiresAll();
```

The call uses the signature and defaults documented above.

## See also

- [`MultipleIterator`](thp:std.spl.MultipleIterator)
