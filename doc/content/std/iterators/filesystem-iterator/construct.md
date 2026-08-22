---
kind: method
id: std.spl.FilesystemIterator::__construct
title: FilesystemIterator::__construct
summary:
  Opens $directory. Keys are full pathnames and values are SplFileInfo objects,
  giving the iterator one stable result type.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: directory
    type: string
    description: Directory to open.
  - name: skip_dots
    type: bool
    description: Omit . and .. entries when true.
    default: "true"
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
owner: std.spl.FilesystemIterator
visibility: public
modifiers: []
---

[`FilesystemIterator`](thp:std.spl.FilesystemIterator)`::__construct()` opens $directory. Keys are full pathnames and values are SplFileInfo objects, giving the iterator one stable result type.

## Behavior

Opens $directory. Keys are full pathnames and values are SplFileInfo objects, giving the iterator one stable result type.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new FilesystemIterator($directory);
```

The call uses the signature and defaults documented above.

## See also

- [`FilesystemIterator`](thp:std.spl.FilesystemIterator)
