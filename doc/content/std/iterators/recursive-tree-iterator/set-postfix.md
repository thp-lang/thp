---
kind: method
id: std.spl.RecursiveTreeIterator::setPostfix
title: RecursiveTreeIterator::setPostfix
summary: Sets text appended to every formatted line.
name: setPostfix
order: 3
typeParameters: []
parameters:
  - name: postfix
    type: string
    description: Text appended to each formatted entry.
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

[`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)`::setPostfix()` sets text appended to every formatted line.

## Behavior

Sets text appended to every formatted line.

This operation may update receiver or resource state; the change is observable by later calls.

## Example

```thp
$instance->setPostfix($postfix);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)
