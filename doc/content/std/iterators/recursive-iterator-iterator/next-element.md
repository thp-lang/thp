---
kind: method
id: std.spl.RecursiveIteratorIterator::nextElement
title: RecursiveIteratorIterator::nextElement
summary: Runs after a value is selected for output.
name: nextElement
order: 9
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

[`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)`::nextElement()` runs after a value is selected for output.

## Behavior

Runs after a value is selected for output.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance->nextElement();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveIteratorIterator`](thp:std.spl.RecursiveIteratorIterator)
