---
kind: method
id: std.spl.SplFileObject::eof
title: SplFileObject::eof
summary: Reports whether the stream reached end of file.
name: eof
order: 2
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the stream reached end of file.
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
owner: std.spl.SplFileObject
visibility: public
modifiers: []
---

[`SplFileObject`](thp:std.spl.SplFileObject)`::eof()` reports whether the stream reached end of file.

## Behavior

Reports whether the stream reached end of file.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->eof();
```

The call uses the signature and defaults documented above.

## See also

- [`SplFileObject`](thp:std.spl.SplFileObject)
