---
kind: method
id: std.spl.RecursiveIteratorIterator::beginChildren
title: RecursiveIteratorIterator::beginChildren
summary: Runs before descending into child entries.
name: beginChildren
order: 7
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
availability: implemented
notice: This experimental traversal member is implemented for recursive cursors.
version: "0.1"
owner: std.spl.RecursiveIteratorIterator
visibility: public
modifiers: []
---

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::beginChildren()` runs before descending into child entries.

## Behavior

Runs before descending into child entries.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->beginChildren();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
