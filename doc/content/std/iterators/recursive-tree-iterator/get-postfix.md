---
kind: method
id: std.spl.RecursiveTreeIterator::getPostfix
title: RecursiveTreeIterator::getPostfix
summary: Returns the configured postfix.
name: getPostfix
order: 6
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the configured postfix.
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

[`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)`::getPostfix()` returns the configured postfix.

## Behavior

Returns the configured postfix.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getPostfix();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)
