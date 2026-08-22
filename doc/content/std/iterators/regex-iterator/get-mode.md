---
kind: method
id: std.spl.RegexIterator::getMode
title: RegexIterator::getMode
summary: Returns the immutable transformation mode.
name: getMode
order: 2
typeParameters: []
parameters: []
returns:
  type: int
  description: Returns the immutable transformation mode.
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
owner: std.spl.RegexIterator
visibility: public
modifiers: []
---

[`RegexIterator`](thp:std.spl.RegexIterator)`::getMode()` returns the immutable transformation mode.

## Behavior

Returns the immutable transformation mode.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getMode();
```

The call uses the signature and defaults documented above.

## See also

- [`RegexIterator`](thp:std.spl.RegexIterator)
