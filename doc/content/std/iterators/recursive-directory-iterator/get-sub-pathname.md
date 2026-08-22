---
kind: method
id: std.spl.RecursiveDirectoryIterator::getSubPathname
title: RecursiveDirectoryIterator::getSubPathname
summary: Returns the current relative child pathname.
name: getSubPathname
order: 3
typeParameters: []
parameters: []
returns:
  type: string
  description: Returns the current relative child pathname.
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

[`RecursiveDirectoryIterator`](thp:std.spl.RecursiveDirectoryIterator)`::getSubPathname()` returns the current relative child pathname.

## Behavior

Returns the current relative child pathname.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getSubPathname();
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveDirectoryIterator`](thp:std.spl.RecursiveDirectoryIterator)
