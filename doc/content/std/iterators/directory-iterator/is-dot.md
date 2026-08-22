---
kind: method
id: std.spl.DirectoryIterator::isDot
title: DirectoryIterator::isDot
summary: Reports whether the entry is . or ...
name: isDot
order: 5
typeParameters: []
parameters: []
returns:
  type: bool
  description: Reports whether the entry is . or ...
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: proposed
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.spl.DirectoryIterator
visibility: public
modifiers: []
---

[`DirectoryIterator`](thp:std.spl.DirectoryIterator)`::isDot()` reports whether the entry is . or ...

## Behavior

Reports whether the entry is . or ...

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->isDot();
```

The call uses the signature and defaults documented above.

## See also

- [`DirectoryIterator`](thp:std.spl.DirectoryIterator)
