---
kind: method
id: std.spl.RecursiveIteratorIterator::setMaxDepth
title: RecursiveIteratorIterator::setMaxDepth
summary: Sets the maximum traversal depth; -1 removes the limit.
name: setMaxDepth
order: 10
typeParameters: []
parameters:
  - name: max_depth
    type: int
    description: Maximum traversal depth; -1 removes the limit.
    default: "-1"
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
owner: std.spl.RecursiveIteratorIterator
visibility: public
modifiers: []
---

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::setMaxDepth()` sets the maximum traversal depth; -1 removes the limit.

## Behavior

Sets the maximum traversal depth; -1 removes the limit.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setMaxDepth();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
