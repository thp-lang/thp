---
kind: method
id: std.spl.RecursiveIteratorIterator::beginIteration
title: RecursiveIteratorIterator::beginIteration
summary: Runs once before the first pull.
name: beginIteration
order: 5
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

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::beginIteration()` runs once before the first pull.

## Behavior

Runs once before the first pull.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->beginIteration();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
