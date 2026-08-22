---
kind: method
id: std.spl.FileLockResult::wouldBlock
title: FileLockResult::wouldBlock
summary:
  Returns true when a non-blocking request failed because another process held an
  incompatible lock. It is false for successful requests and other failures.
name: wouldBlock
order: 2
typeParameters: []
parameters: []
returns:
  type: bool
  description:
    Returns true when a non-blocking request failed because another process
    held an incompatible lock. It is false for successful requests and other failures.
errors:
  - description:
      Underlying I/O failures follow the return sentinel shown in the signature
      or propagate as the experimental THP I/O failure where no sentinel is available.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.FileLockResult
visibility: public
modifiers: []
---

[`FileLockResult`](thp:std.spl.FileLockResult)`::wouldBlock()` returns true when a non-blocking request failed because another process held an incompatible lock. It is false for successful requests and other failures.

## Behavior

Returns true when a non-blocking request failed because another process held an incompatible lock. It is false for successful requests and other failures.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->wouldBlock();
```

The call uses the signature and defaults documented above.

## See also

- [`FileLockResult`](thp:std.spl.FileLockResult)
