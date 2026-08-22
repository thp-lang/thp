---
kind: class
id: std.spl.SplFileObject
title: SplFileObject
summary: Combines a file stream with line-oriented iteration.
name: SplFileObject
module: filesystem
typeParameters: []
parent:
  id: std.spl.SplFileInfo
interfaces:
  - id: std.spl.SeekableIterator
    arguments:
      - int
      - string
constants:
  - name: DROP_NEW_LINE
    type: int
    description: Removes the trailing line ending.
  - name: READ_AHEAD
    type: int
    description: Buffers the next line before cursor advancement reaches it.
  - name: SKIP_EMPTY
    type: int
    description: Skips empty lines.
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`SplFileObject` combines a file stream with line-oriented iteration.

## Construction

| Method                                                    | Description                                                                                                 |
| --------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.SplFileObject::__construct) | Opens the path with the requested stream mode and retains ownership of that stream for the object lifetime. |

## Behavior

Direct iteration is line-oriented and yields `string`. `csvRows()` provides a
separate iterator with a stable `vector<?string>` element type. Seeking by
iterator position addresses line numbers, while `fseek()` addresses byte
offsets.

## Differences from PHP

PHP exposes CSV iteration through `READ_CSV`, writes lock status through an
output parameter, and maintains a mutable iterator cursor. THP provides
[`csvRows()`](thp:std.spl.SplFileObject::csvRows), returns a typed
[`FileLockResult`](thp:std.spl.FileLockResult) from [`flock()`](thp:std.spl.SplFileObject::flock), and
uses the pull-based `Iterator` contract.

## Errors

Construction fails when the stream cannot be opened. Individual stream operations use the documented `false`, integer status, or nullable results; additional I/O failures may propagate without a finalized THP error class.

## Example

```thp
$file = new SplFileObject("./data/users.csv");
foreach ($file->csvRows() as $row) {
    if ($row[0] !== null) {
        print($row[0]);
    }
}
```

Each yielded value is a parsed CSV row.

## See also

- [SPL file handling](thp:std.filesystem)
- [PHP `SplFileObject`](https://www.php.net/manual/en/class.splfileobject.php)
- [`SplFileInfo`](thp:std.spl.SplFileInfo)
