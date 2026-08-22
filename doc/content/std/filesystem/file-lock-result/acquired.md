---
kind: method
id: std.spl.FileLockResult::acquired
title: FileLockResult::acquired
summary: Returns true when the requested lock was acquired.
name: acquired
order: 1
typeParameters: []
parameters: []
returns:
  type: bool
  description: Returns true when the requested lock was acquired.
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

[`FileLockResult`](thp:std.spl.FileLockResult)`::acquired()` returns true when the requested lock was acquired.

## Behavior

Returns true when the requested lock was acquired.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->acquired();
```

The call uses the signature and defaults documented above.

## See also

- [`FileLockResult`](thp:std.spl.FileLockResult)
