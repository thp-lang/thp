---
kind: class
id: std.spl.DirectoryIterator
title: DirectoryIterator
summary: Iterates entries in one filesystem directory.
name: DirectoryIterator
module: iterators
typeParameters: []
interfaces:
  - id: std.spl.SeekableIterator
    arguments:
      - int
      - SplFileInfo
  - id: std.baseTypes.Stringable
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`DirectoryIterator` iterates entries in one filesystem directory.

## Construction

| Method                                                        | Description                                                       |
| ------------------------------------------------------------- | ----------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.DirectoryIterator::__construct) | Opens $directory and prepares a cursor iterator over its entries. |

## Behavior

Entries follow the directory stream’s platform-dependent order and may include
`.` and `..`. Each yielded `SplFileInfo` describes one entry.

## Errors

Construction validates the parameters shown above. Cursor operations propagate failures from the wrapped iterator, callback, pattern engine, or filesystem when that dependency is present; each member page identifies the applicable source. Concrete THP error classes remain unsettled.

## Example

```thp
$entries = new DirectoryIterator("./assets");
foreach ($entries as $entry) {
    if (!$entry->isDot()) {
        print($entry->getFilename());
    }
}
```

Dot entries are explicitly skipped by the caller.

## See also

- [SPL iterators](thp:std.iterators)
- [PHP `DirectoryIterator`](https://www.php.net/manual/en/class.directoryiterator.php)
