---
kind: method
id: std.spl.RecursiveIteratorIterator::getMaxDepth
title: RecursiveIteratorIterator::getMaxDepth
summary: Returns the maximum depth, or false when unlimited.
name: getMaxDepth
order: 11
typeParameters: []
parameters: []
returns:
  type: int|false
  description: Returns the maximum depth, or false when unlimited.
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
owner: std.spl.RecursiveIteratorIterator
visibility: public
modifiers: []
---

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::getMaxDepth()` returns the maximum depth, or false when unlimited.

## Behavior

Returns the maximum depth, or false when unlimited.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getMaxDepth();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
