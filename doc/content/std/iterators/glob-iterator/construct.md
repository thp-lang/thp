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
    description: Reserved for future nested traversal; has no effect on current final-component matches.
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
availability: partial
notice: Final-component * and ? patterns execute. Nested patterns and character
  classes remain proposed.
version: "0.1"
owner: std.spl.GlobIterator
visibility: public
modifiers: []
---

[`GlobIterator`](thp:std.spl.GlobIterator)`::__construct()` expands $pattern and iterates matching paths as the same typed key-value pairs returned by FilesystemIterator.

## Behavior

Expands `*` and `?` in the final path component and iterates matching paths as
`string` keys and `SplFileInfo` values. Unsupported pattern forms raise
`ValueError`; inaccessible directories raise `OpenStreamException`.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$instance = new GlobIterator($pattern);
```

The call uses the signature and defaults documented above.

## See also

- [`GlobIterator`](thp:std.spl.GlobIterator)
