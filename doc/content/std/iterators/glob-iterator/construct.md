---
kind: method
id: std.spl.GlobIterator::__construct
title: GlobIterator::__construct
summary:
  Expands $pattern and iterates matching paths as the same typed key-value pairs
  returned by FilesystemIterator.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: pattern
    type: string
    description: Pattern used by the operation.
  - name: follow_symlinks
    type: bool
    description: Permit traversal through symbolic links when true.
    default: "false"
  - name: unix_paths
    type: bool
    description: Use / as the separator in returned paths on every platform.
    default: "false"
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Construction fails when an argument violates the documented contract or an
      underlying resource cannot be created. Concrete THP error classes remain
      experimental unless named above.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.GlobIterator
visibility: public
modifiers: []
---

[`GlobIterator`](thp:std.spl.GlobIterator)`::__construct()` expands $pattern and iterates matching paths as the same typed key-value pairs returned by FilesystemIterator.

## Behavior

Expands $pattern and iterates matching paths as the same typed key-value pairs returned by FilesystemIterator.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new GlobIterator($pattern);
```

The call uses the signature and defaults documented above.

## See also

- [`GlobIterator`](thp:std.spl.GlobIterator)
