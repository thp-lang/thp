---
kind: method
id: std.spl.RegexIterator::getRegex
title: RegexIterator::getRegex
summary: Returns the regular-expression pattern.
name: getRegex
order: 3
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the regular-expression pattern.
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

[`RegexIterator`](thp:std.spl.RegexIterator)`::getRegex()` returns the regular-expression pattern.

## Behavior

Returns the regular-expression pattern.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getRegex();
```

The call uses the signature and defaults documented above.

## See also

- [`RegexIterator`](thp:std.spl.RegexIterator)
