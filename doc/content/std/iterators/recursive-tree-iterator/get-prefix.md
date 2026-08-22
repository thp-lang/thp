---
kind: method
id: std.spl.RecursiveTreeIterator::getPrefix
title: RecursiveTreeIterator::getPrefix
summary: Returns the prefix for the next formatted line.
name: getPrefix
order: 2
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the prefix for the next formatted line.
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

[`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)`::getPrefix()` returns the prefix for the next formatted line.

## Behavior

Returns the prefix for the next formatted line.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getPrefix();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)
