---
kind: class
id: std.spl.SplFileInfo
title: SplFileInfo
summary: Represents metadata and operations for one filesystem path.
name: SplFileInfo
module: filesystem
typeParameters: []
interfaces:
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

`SplFileInfo` represents metadata and operations for one filesystem path.

## Construction

| Method                                                  | Description                                                             |
| ------------------------------------------------------- | ----------------------------------------------------------------------- |
| [`__construct()`](thp:std.spl.SplFileInfo::__construct) | Stores the supplied path without requiring that the path already exist. |

## Behavior

Metadata calls observe the filesystem when invoked and can therefore change between calls. Path interpretation, permissions, owners, groups, and timestamps remain platform-dependent.

## Errors

Metadata methods return `false` where shown when information is unavailable. Opening a file or selecting an incompatible subclass fails; concrete THP error classes are unsettled.

## Example

```thp
$info = new SplFileInfo("./config/app.thp");
if ($info->isFile()) {
    print($info->getBasename(".thp"));
}
```

For that path, the basename without the suffix is `"app"`.

## See also

- [SPL file handling](thp:std.filesystem)
- [PHP `SplFileInfo`](https://www.php.net/manual/en/class.splfileinfo.php)
