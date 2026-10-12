---
kind: class
id: std.spl.GlobIterator
title: GlobIterator
summary: Iterates filesystem entries matched by a glob pattern.
name: GlobIterator
module: iterators
typeParameters: []
interfaces:
  - id: std.baseTypes.Iterator
    arguments:
      - string
      - SplFileInfo
  - id: std.baseTypes.Countable
constants: []
properties: []
status: experimental
availability: partial
notice: Final-component * and ? patterns, iteration, and count() execute. Nested
  patterns, character classes, and inherited FilesystemIterator behavior remain
  proposed.
version: "0.1"
---

`GlobIterator` iterates filesystem entries matched by a glob pattern.

## Construction

| Method                                                   | Description                                                                         |
| -------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.GlobIterator::__construct) | Expands $pattern and iterates the same typed key-value pairs as FilesystemIterator. |

## Behavior

The constructor matches `*` and `?` in the final path component. It captures
matching entries when constructed. The optional booleans retain their declared
signatures; recursive traversal is not implemented.

## Differences from PHP

PHP accepts the inherited `FilesystemIterator` integer flags. THP keeps the
result type fixed and exposes only `$follow_symlinks` and `$unix_paths` as
constructor booleans.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$sources = new GlobIterator("./src/*.thp");
print($sources->count());
```

Matches are captured when the iterator is constructed, so `count()` remains stable for that iterator even if the directory later changes.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `GlobIterator`](https://www.php.net/manual/en/class.globiterator.php)
