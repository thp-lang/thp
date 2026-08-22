---
kind: class
id: std.spl.RecursiveDirectoryIterator
title: RecursiveDirectoryIterator
summary: Traverses a directory and exposes subdirectories as children.
name: RecursiveDirectoryIterator
module: iterators
typeParameters: []
interfaces:
  - id: std.spl.RecursiveIterator
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

`RecursiveDirectoryIterator` traverses a directory and exposes subdirectories as children.

## Construction

| Method                                                                 | Description                                                                                                                              |
| ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.RecursiveDirectoryIterator::__construct) | Opens the directory and yields a RecursiveEntry for each child. Directory entries expose a child iterator unless link policy forbids it. |

## Behavior

Directories become the yielded entry's child iterator. Symbolic links are not
followed unless enabled because link cycles can make traversal unbounded.

## Differences from PHP

PHP uses `FilesystemIterator` flags and separate child-inspection methods.
THP uses explicit booleans and carries an optional child iterator directly in
each `RecursiveEntry<SplFileInfo>`; full pathnames are exposed as cursor keys.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$tree = new RecursiveDirectoryIterator("./src");
$files = new RecursiveIteratorIterator<string, SplFileInfo>($tree);
```

The recursive traversal can visit files below `./src`.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `RecursiveDirectoryIterator`](https://www.php.net/manual/en/class.recursivedirectoryiterator.php)
