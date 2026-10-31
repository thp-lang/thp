---
kind: module
id: std.filesystem
title: Filesystem
summary: Object-oriented filesystem metadata, files, and temporary files.
module: filesystem
order: 50
status: experimental
availability: partial
notice: SplFileInfo path accessors and common metadata methods execute. The
  read-only SplFileObject line iterator is partial; temporary-file objects,
  CSV, locking, and advanced metadata operations remain proposed.
---

| Class                                                | Purpose                                  |
| ---------------------------------------------------- | ---------------------------------------- |
| [`FileLockResult`](thp:std.spl.FileLockResult)       | Reports a file-lock request outcome.     |
| [`SplFileInfo`](thp:std.spl.SplFileInfo)             | Describes one filesystem path.           |
| [`SplFileObject`](thp:std.spl.SplFileObject)         | Reads, writes, and iterates a file.      |
| [`SplTempFileObject`](thp:std.spl.SplTempFileObject) | Provides a memory-backed temporary file. |

## See also

- [SPL reference](thp:std.dataStructures)
- [THP resources and streams](thp:guide.languageResourcesAndStreams)
- [PHP SPL file handling](https://www.php.net/manual/en/spl.files.php)
