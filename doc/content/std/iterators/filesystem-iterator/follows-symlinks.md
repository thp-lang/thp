---
kind: method
id: std.spl.FilesystemIterator::followsSymlinks
title: FilesystemIterator::followsSymlinks
summary: Reports whether symbolic links may be followed.
name: followsSymlinks
order: 3
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether symbolic links may be followed.
errors:
  - description:
      Underlying I/O failures follow the return sentinel shown in the signature
      or propagate as the experimental THP I/O failure where no sentinel is available.
related: []
status: experimental
availability: implemented
notice: Reports the follow_symlinks option used at construction.
version: "0.1"
owner: std.spl.FilesystemIterator
visibility: public
modifiers: []
---

[`FilesystemIterator`](thp:std.spl.FilesystemIterator)`::followsSymlinks()` reports whether symbolic links may be followed.

## Behavior

Reports whether symbolic links may be followed.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->followsSymlinks();
```

The call uses the signature and defaults documented above.

## See also

- [`FilesystemIterator`](thp:std.spl.FilesystemIterator)
