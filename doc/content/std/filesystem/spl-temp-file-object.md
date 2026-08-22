---
kind: class
id: std.spl.SplTempFileObject
title: SplTempFileObject
summary: Provides a temporary stream that spills from memory to disk.
name: SplTempFileObject
module: filesystem
typeParameters: []
parent:
  id: std.spl.SplFileObject
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This PHP-inspired class contract is proposed and is not implemented in this
  repository. Types, inheritance, and failure behavior may change.
version: "0.1"
---

`SplTempFileObject` provides a temporary stream that spills from memory to disk.

## Construction

| Method                                                        | Description                                                                                                                                                                                                |
| ------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.SplTempFileObject::__construct) | Creates a temporary stream. Positive values keep up to that many bytes in memory before spilling to a temporary file, zero uses a temporary file immediately, and negative values keep all data in memory. |

## Behavior

The object inherits file reading, writing, CSV, locking, and iteration operations. Temporary storage is released when the stream closes.

## Temporary-stream limitations

The inherited stream-reading and stream-writing methods remain usable.
Filesystem-path metadata is not meaningful for the temporary URI:
`isReadable()` and `isWritable()` report `false`, path-stat methods can
fail, and `flock()` reports an unsuccessful lock unless the backing stream
supports operating-system locks.

## Errors

Construction or spill-to-disk throws `IoException` when temporary storage is
unavailable.

## Example

```thp
$buffer = new SplTempFileObject();
$buffer->fwrite("report\n");
$buffer->seek(0);
$line = $buffer->fgets();
```

`$line` contains the text written to the temporary stream.

## See also

- [SPL file handling](thp:std.filesystem)
- [THP resources and streams](thp:guide.languageResourcesAndStreams)
- [PHP `SplTempFileObject`](https://www.php.net/manual/en/class.spltempfileobject.php)
- [`SplFileObject`](thp:std.spl.SplFileObject)
