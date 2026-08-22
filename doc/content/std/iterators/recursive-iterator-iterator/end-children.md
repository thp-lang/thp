---
kind: method
id: std.spl.RecursiveIteratorIterator::endChildren
title: RecursiveIteratorIterator::endChildren
summary: Runs after a child iterator is exhausted.
name: endChildren
order: 8
typeParameters: []
parameters: []
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

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::endChildren()` runs after a child iterator is exhausted.

## Behavior

Runs after a child iterator is exhausted.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->endChildren();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
