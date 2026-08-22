---
kind: method
id: std.spl.RecursiveDirectoryIterator::__construct
title: RecursiveDirectoryIterator::__construct
summary:
  Opens the directory and yields a RecursiveEntry for each child. Directory
  entries expose a child iterator unless link policy forbids it.
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
owner: std.spl.RecursiveDirectoryIterator
visibility: public
modifiers: []
---

[`RecursiveDirectoryIterator`](thp:std.spl.RecursiveDirectoryIterator)`::__construct()` opens the directory and yields a RecursiveEntry for each child. Directory entries expose a child iterator unless link policy forbids it.

## Behavior

Opens the directory and yields a RecursiveEntry for each child. Directory entries expose a child iterator unless link policy forbids it.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new RecursiveDirectoryIterator($directory);
```

The call uses the signature and defaults documented above.

## See also

- [`RecursiveDirectoryIterator`](thp:std.spl.RecursiveDirectoryIterator)
