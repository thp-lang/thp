---
kind: method
id: std.spl.RecursiveTreeIterator::getEntry
title: RecursiveTreeIterator::getEntry
summary: Returns the formatted entry text without prefix or postfix.
name: getEntry
order: 5
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the formatted entry text without prefix or postfix.
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

[`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)`::getEntry()` returns the formatted entry text without prefix or postfix.

## Behavior

Returns the formatted entry text without prefix or postfix.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getEntry();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveTreeIterator`](thp:std.spl.RecursiveTreeIterator)
