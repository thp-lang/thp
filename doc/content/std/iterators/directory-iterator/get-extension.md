---
kind: method
id: std.spl.DirectoryIterator::getExtension
title: DirectoryIterator::getExtension
summary: Returns the filename extension.
name: getExtension
order: 3
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the filename extension.
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

[`DirectoryIterator`](thp:std.spl.DirectoryIterator)`::getExtension()` returns the filename extension.

## Behavior

Returns the filename extension.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getExtension();
```

The call uses the signature and defaults documented above.

## See also

- [`DirectoryIterator`](thp:std.spl.DirectoryIterator)
