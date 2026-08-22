---
kind: method
id: std.spl.FilesystemIterator::skipsDots
title: FilesystemIterator::skipsDots
summary: Reports whether dot entries are omitted.
name: skipsDots
order: 2
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether dot entries are omitted.
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
owner: std.spl.FilesystemIterator
visibility: public
modifiers: []
---

[`FilesystemIterator`](thp:std.spl.FilesystemIterator)`::skipsDots()` reports whether dot entries are omitted.

## Behavior

Reports whether dot entries are omitted.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->skipsDots();
```

The call uses the signature and defaults documented above.

## See also

- [`FilesystemIterator`](thp:std.spl.FilesystemIterator)
