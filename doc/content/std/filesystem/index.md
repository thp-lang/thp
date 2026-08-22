---
kind: module
id: std.filesystem
title: Filesystem
summary: Object-oriented filesystem metadata, files, and temporary files.
module: filesystem
order: 50
status: experimental
availability: proposed
notice: These PHP-inspired classes are not implemented. Paths, streams, platform
  differences, and integration with THP's proposed typed stream model remain design
  work.
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
