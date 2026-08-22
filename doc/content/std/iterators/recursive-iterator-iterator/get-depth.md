---
kind: method
id: std.spl.RecursiveIteratorIterator::getDepth
title: RecursiveIteratorIterator::getDepth
summary: Returns the depth of the most recently yielded value.
name: getDepth
order: 2
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the depth of the most recently yielded value.
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

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::getDepth()` returns the depth of the most recently yielded value.

## Behavior

Returns the depth of the most recently yielded value.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getDepth();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
