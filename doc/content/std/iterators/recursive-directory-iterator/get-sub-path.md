---
kind: method
id: std.spl.RecursiveDirectoryIterator::getSubPath
title: RecursiveDirectoryIterator::getSubPath
summary: Returns the path relative to the root.
name: getSubPath
order: 2
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the path relative to the root.
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
owner: std.spl.RecursiveDirectoryIterator
visibility: public
modifiers: []
---

[`RecursiveDirectoryIterator`](thp:std.spl.RecursiveDirectoryIterator)`::getSubPath()` returns the path relative to the root.

## Behavior

Returns the path relative to the root.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getSubPath();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveDirectoryIterator`](thp:std.spl.RecursiveDirectoryIterator)
