---
kind: method
id: std.spl.DirectoryIterator::getFilename
title: DirectoryIterator::getFilename
summary: Returns the entry filename.
name: getFilename
order: 2
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the entry filename.
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

[`DirectoryIterator`](thp:std.spl.DirectoryIterator)`::getFilename()` returns the entry filename.

## Behavior

Returns the entry filename.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getFilename();
```

The call uses the signature and defaults documented above.

## See also

- [`DirectoryIterator`](thp:std.spl.DirectoryIterator)
