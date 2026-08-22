---
kind: method
id: std.spl.DirectoryIterator::getBasename
title: DirectoryIterator::getBasename
summary: Returns the basename after removing an optional suffix.
name: getBasename
order: 4
typeParameters: []
parameters:
  - name: suffix
    type: string
    description: Optional suffix removed from the returned basename.
    default: '""'
returns:
  type: string
  description: Returns the basename after removing an optional suffix.
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

[`DirectoryIterator`](thp:std.spl.DirectoryIterator)`::getBasename()` returns the basename after removing an optional suffix.

## Behavior

Returns the basename after removing an optional suffix.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getBasename();
```

The call uses the signature and defaults documented above.

## See also

- [`DirectoryIterator`](thp:std.spl.DirectoryIterator)
