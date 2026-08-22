---
kind: method
id: std.spl.RecursiveTreeIterator::setPrefixPart
title: RecursiveTreeIterator::setPrefixPart
summary: Replaces one branch-prefix segment.
name: setPrefixPart
order: 4
typeParameters: []
parameters:
  - name: part
    type: int
    description: Prefix segment selected below.
  - name: value
    type: string
    description: Value consumed or stored by the operation.
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
owner: std.spl.RecursiveTreeIterator
visibility: public
modifiers: []
---

[`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)`::setPrefixPart()` replaces one branch-prefix segment.

## Behavior

Replaces one branch-prefix segment.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setPrefixPart($part, $value);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)
