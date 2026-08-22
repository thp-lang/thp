---
kind: class
id: std.spl.FileLockResult
title: FileLockResult
summary: Reports whether a non-blocking file-lock request succeeded.
name: FileLockResult
module: filesystem
typeParameters: []
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice:
  This result replaces PHP's by-reference flock() output in the intended THP API.
  It is not implemented in this checkout.
version: "0.1"
---

This is a final class.

`FileLockResult` reports the outcome of one file-lock request.

## Behavior

The immutable result replaces an output reference and keeps the two status
values paired.

## Example

```thp
$result: FileLockResult = $file->flock(LOCK_EX | LOCK_NB);
if ($result->wouldBlock()) {
    echo "busy";
}
```

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
