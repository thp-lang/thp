---
kind: method
id: std.spl.RecursiveIteratorIterator::endIteration
title: RecursiveIteratorIterator::endIteration
summary: Runs once after exhaustion.
name: endIteration
order: 6
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

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::endIteration()` runs once after exhaustion.

## Behavior

Runs once after exhaustion.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->endIteration();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
