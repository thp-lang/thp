---
kind: method
id: std.spl.SplTempFileObject::__construct
title: SplTempFileObject::__construct
summary:
  Creates a temporary stream. Positive values keep up to that many bytes in
  memory before spilling to a temporary file, zero uses a temporary file immediately,
  and negative values keep all data in memory.
name: __construct
order: 1
typeParameters: []
parameters:
  - name: max_memory
    type: int
    description:
      Positive spill threshold in bytes; zero selects disk-only storage and a
      negative value selects memory-only storage.
    default: "2097152"
returns:
  type: void
  description: This callable does not return a value.
errors:
  - description:
      Construction can fail when the runtime cannot create required temporary
      storage. Every integer value has defined storage semantics, including zero and
      negative values.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.SplTempFileObject
visibility: public
modifiers: []
---

[`SplTempFileObject`](thp:std.spl.SplTempFileObject)`::__construct()` creates a temporary stream. Positive
values keep up to that many bytes in memory before spilling to a temporary file,
zero uses a temporary file immediately, and negative values keep all data in
memory.

## Behavior

Creates a temporary stream. Positive values keep up to that many bytes in memory before spilling to a temporary file, zero uses a temporary file immediately, and negative values keep all data in memory.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$memory_only = new SplTempFileObject(-1);
```

The call uses the signature and defaults documented above.

## See also

- [`SplTempFileObject`](thp:std.spl.SplTempFileObject)
