---
kind: class
id: std.spl.FilesystemIterator
title: FilesystemIterator
summary: Iterates directory entries as path and file-information pairs.
name: FilesystemIterator
module: iterators
typeParameters: []
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - string
      - SplFileInfo
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`FilesystemIterator` iterates directory entries as stable keyed values.

## Construction

| Method                                                         | Description                                                                                                               |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.FilesystemIterator::__construct) | Opens $directory. Keys are full pathnames and values are SplFileInfo objects, giving the iterator one stable result type. |

## Behavior

The cursor exposes a full-path `string` key and the corresponding
`SplFileInfo` value without constructing an entry object.
Constructor options control dot entries and symbolic links without changing the
result type.

## Differences from PHP

PHP combines key type, value type, dot-entry handling, symbolic-link handling,
and `UNIX_PATHS` in an integer flag mask. THP always exposes
`Iterator<string, SplFileInfo>` and represents the remaining choices as
`$skip_dots`, `$follow_symlinks`, and `$unix_paths` booleans.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$files = new FilesystemIterator("./assets");
foreach ($files as $path => $info) {
    print($path);
}
```

The default mode skips dot entries and yields file information keyed by path.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `FilesystemIterator`](https://www.php.net/manual/en/class.filesystemiterator.php)
