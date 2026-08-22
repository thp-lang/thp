---
kind: method
id: std.spl.DirectoryIterator::__toString
title: DirectoryIterator::__toString
summary: Returns the entry pathname.
name: __toString
order: 7
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the entry pathname.
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
owner: std.spl.DirectoryIterator
visibility: public
modifiers: []
---

[`DirectoryIterator`](thp:std.spl.DirectoryIterator)`::__toString()` returns the entry pathname.

## Behavior

Returns the entry pathname.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->__toString();
```

The call uses the signature and defaults documented above.

## See also

- [`DirectoryIterator`](thp:std.spl.DirectoryIterator)
